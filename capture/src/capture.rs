//! Packet capture module with simulation and real pcap support.
//! 
//! This module provides a unified interface for packet capture, supporting both:
//! - **Simulation mode**: Generates synthetic packets for testing (default)
//! - **Real capture**: Uses pcap library to capture from network interfaces
//!
//! ## Architecture
//! 
//! ```text
//! +-------------------+     +----------------------+     +------------------+
//! | PacketSource     | --> | PacketProcessor      | --> | ClassifierOutput |
//! | (trait)          |     | (feature extraction) |     | (classified)     |
//! +-------------------+     +----------------------+     +------------------+
//! |                   |                           |
//! +-------------------+                           +
//! | SimulationSource |                           +------------------+
//! | PcapSource       |                           | RuleBasedClassifier|
//! +-------------------+                           +------------------+
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use capture::capture::{start_capture, CaptureConfig};
//!
//! // Simulation mode (default)
//! let config = CaptureConfig::simulation();
//! start_capture(config, tx).await?;
//!
//! // Real pcap capture
//! let config = CaptureConfig::pcap("eth0");
//! start_capture(config, tx).await?;
//! ```

use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::sync::Arc;

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for packet capture source.
/// 
/// Specifies whether to use simulation mode or real pcap capture,
/// along with relevant parameters for each mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Capture mode: "simulation" or "pcap"
    pub mode: String,
    /// Network interface for pcap mode (e.g., "eth0", "lo0")
    #[serde(default)]
    pub interface: Option<String>,
    /// BPF filter for pcap (e.g., "tcp or udp")
    #[serde(default)]
    pub filter: Option<String>,
    /// Packets per second for simulation mode
    #[serde(default = "default_pps")]
    pub simulation_pps: u32,
    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,
}

fn default_pps() -> u32 { 10000 }

impl CaptureConfig {
    /// Create simulation mode configuration with default settings.
    /// 
    /// # Example
    /// ```rust
    /// let config = CaptureConfig::simulation();
    /// // Generates ~10,000 packets/second
    /// ```
    pub fn simulation() -> Self {
        Self {
            mode: "simulation".to_string(),
            interface: None,
            filter: None,
            simulation_pps: 10000,
            verbose: false,
        }
    }

    /// Create pcap mode configuration for real packet capture.
    /// 
    /// # Arguments
    /// * `interface` - Network interface name (e.g., "eth0", "lo0", "en0")
    /// 
    /// # Example
    /// ```rust
    /// let config = CaptureConfig::pcap("lo0");
    /// ```
    pub fn pcap(interface: &str) -> Self {
        Self {
            mode: "pcap".to_string(),
            interface: Some(interface.to_string()),
            filter: Some("tcp or udp".to_string()),
            simulation_pps: 10000,
            verbose: false,
        }
    }
}

// ============================================================================
// Data Structures
// ============================================================================

/// Extracted features from a network packet.
/// 
/// These features are used as input to the classifier.
/// All numeric values are normalized for ML inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketFeatures {
    /// Source port (0-65535)
    pub src_port: u16,
    /// Destination port (0-65535)
    pub dst_port: u16,
    /// IP protocol number (6=TCP, 17=UDP, 1=ICMP)
    pub protocol: u8,
    /// Total packet size in bytes
    pub packet_size: u32,
    /// Payload size (bytes after transport header)
    pub payload_size: u32,
    /// TCP flags (if TCP packet)
    pub tcp_flags: u8,
    /// Timestamp in microseconds since epoch
    pub timestamp_us: u64,
}

impl PacketFeatures {
    /// Parse raw packet bytes to extract features.
    /// 
    /// Supports Ethernet + IPv4 + TCP/UDP/ICMP packets.
    /// Returns None if packet is malformed or unsupported.
    /// 
    /// # Packet Format Expected
    /// ```text
    /// [Ethernet Header: 14 bytes]
    ///   - Destination MAC: 6 bytes
    ///   - Source MAC: 6 bytes  
    ///   - EtherType: 2 bytes (0x0800 = IPv4)
    /// [IPv4 Header: 20+ bytes]
    ///   - IHL: 4 bits (header length in 32-bit words)
    ///   - Protocol: 8 bits (6=TCP, 17=UDP, 1=ICMP)
    ///   - Source IP: 4 bytes
    ///   - Destination IP: 4 bytes
    /// [Transport Header: varies]
    ///   - TCP: 20+ bytes (ports, flags, etc.)
    ///   - UDP: 8 bytes (ports, length)
    /// ```
    pub fn from_slice(data: &[u8], timestamp: Duration) -> Option<Self> {
        // Minimum: Ethernet(14) + IPv4(20) + Transport(4 for ports)
        if data.len() < 38 { return None; }
        
        // Parse Ethernet header
        let eth_type = u16::from_be_bytes([data[12], data[13]]);
        if eth_type != 0x0800 { return None; } // IPv4 only for now
        
        // Parse IPv4 header
        let ip_header_len = ((data[14] & 0x0F) as usize) * 4;
        if data.len() < 14 + ip_header_len + 4 { return None; }
        
        let protocol = data[14 + 9];
        
        // Parse transport layer (TCP/UDP)
        let (src_port, dst_port, payload_size, tcp_flags) = match protocol {
            6 => { // TCP
                let tcp_start = 14 + ip_header_len;
                if data.len() < tcp_start + 20 { return None; }
                let src_p = u16::from_be_bytes([data[tcp_start], data[tcp_start+1]]);
                let dst_p = u16::from_be_bytes([data[tcp_start+2], data[tcp_start+3]]);
                let payload = data.len().saturating_sub(tcp_start + 20);
                let flags = data.get(tcp_start + 13).copied().unwrap_or(0);
                (src_p, dst_p, payload as u32, flags)
            }
            17 => { // UDP
                let udp_start = 14 + ip_header_len;
                if data.len() < udp_start + 8 { return None; }
                let src_p = u16::from_be_bytes([data[udp_start], data[udp_start+1]]);
                let dst_p = u16::from_be_bytes([data[udp_start+2], data[udp_start+3]]);
                let payload = data.len().saturating_sub(udp_start + 8);
                (src_p, dst_p, payload as u32, 0)
            }
            1 => (0, 0, 0, 0), // ICMP
            _ => (0, 0, 0, 0),
        };

        Some(PacketFeatures {
            src_port,
            dst_port,
            protocol,
            packet_size: data.len() as u32,
            payload_size,
            tcp_flags,
            timestamp_us: timestamp.as_micros() as u64,
        })
    }

    /// Convert features to normalized vector for ML inference.
    /// 
    /// Each feature is normalized to [0, 1] range for neural network input.
    pub fn to_vector(&self) -> Vec<f32> {
        vec![
            self.src_port as f32 / 65535.0,
            self.dst_port as f32 / 65535.0,
            self.protocol as f32 / 255.0,
            self.packet_size as f32 / 1500.0,
            self.payload_size as f32 / 1400.0,
            self.tcp_flags as f32 / 255.0,
        ]
    }
}

/// Result of classifying a packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierOutput {
    /// Extracted packet features
    pub features: PacketFeatures,
    /// Class ID (for ML models)
    pub class_id: u8,
    /// Human-readable class name
    pub class_name: String,
    /// Confidence score [0.0, 1.0]
    pub confidence: f32,
    /// Flow-level statistics (optional)
    pub flow_stats: Option<FlowSummary>,
}

/// Summary of a network flow (aggregate of packets between same endpoints).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSummary {
    /// Destination port
    pub dst_port: u16,
    /// Protocol name ("TCP", "UDP", "ICMP")
    pub protocol: String,
    /// Classification result
    pub class_name: String,
    /// Total packets in flow
    pub packet_count: u64,
    /// Total bytes in flow
    pub byte_count: u64,
}

// ============================================================================
// Rule-Based Classifier
// ============================================================================

/// Simple rule-based classifier based on port numbers.
/// 
/// This is the default classifier used when ML model is not available.
/// It maps well-known ports to protocol names with confidence scores.
/// 
/// ## Supported Protocols
/// - HTTP (80), HTTPS (443), SSH (22), FTP (21), DNS (53)
/// - SMTP (25, 587, 465), MySQL (3306), PostgreSQL (5432), Redis (6379)
/// - HTTP alternatives (8080, 8000, 3000)
pub fn classify_port(port: u16) -> (String, f32) {
    match port {
        80 => ("HTTP".to_string(), 0.95),
        443 => ("HTTPS".to_string(), 0.95),
        22 => ("SSH".to_string(), 0.95),
        21 => ("FTP".to_string(), 0.90),
        53 => ("DNS".to_string(), 0.90),
        25 | 587 | 465 => ("SMTP".to_string(), 0.85),
        110 | 995 => ("POP3".to_string(), 0.85),
        143 | 993 => ("IMAP".to_string(), 0.85),
        3306 => ("MySQL".to_string(), 0.90),
        5432 => ("PostgreSQL".to_string(), 0.90),
        6379 => ("Redis".to_string(), 0.90),
        27017 => ("MongoDB".to_string(), 0.90),
        8080 | 8000 | 3000 => ("HTTP-Alt".to_string(), 0.80),
        _ => ("Unknown".to_string(), 0.50),
    }
}

// ============================================================================
// Main Capture Function
// ============================================================================

/// Start packet capture with specified configuration.
/// 
/// This is the main entry point for the capture module. It selects the
/// appropriate capture mode based on configuration and runs until
/// the channel is closed or an error occurs.
/// 
/// # Arguments
/// * `config` - Capture configuration (simulation or pcap)
/// * `tx` - Channel sender for classified packets
/// 
/// # Behavior
/// - **Simulation**: Generates synthetic packets at configured rate
/// - **Pcap**: Captures real packets from network interface
pub async fn start_capture(
    config: CaptureConfig,
    tx: mpsc::Sender<ClassifierOutput>,
) -> anyhow::Result<()> {
    match config.mode.as_str() {
        "simulation" => start_simulation(config, tx).await,
        "pcap" => start_pcap(config, tx).await,
        _ => {
            eprintln!("Unknown capture mode: {}, using simulation", config.mode);
            start_simulation(CaptureConfig::simulation(), tx).await
        }
    }
}

// ============================================================================
// Simulation Mode Implementation
// ============================================================================

/// Simple deterministic RNG for thread-safe simulation.
/// 
/// Uses a linear congruential generator (LCG) - not cryptographically
/// secure but sufficient for packet simulation. Thread-safe because
/// it doesn't use internal mutable state across await points.
struct SimpleRng { state: u64 }

impl SimpleRng {
    fn new(seed: u64) -> Self { Self { state: seed } }
    
    /// Generate next random value.
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state >> 16
    }
    
    /// Generate random value in range [0, max).
    fn range(&mut self, max: usize) -> usize {
        (self.next() as usize) % max
    }
}

/// Run simulation mode - generates synthetic packets.
/// 
/// Uses deterministic RNG to create realistic-looking traffic patterns
/// without requiring network access. Configurable packets/second rate.
async fn start_simulation(
    config: CaptureConfig,
    tx: mpsc::Sender<ClassifierOutput>,
) -> anyhow::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    
    let counter = Arc::new(AtomicU64::new(0));
    let mut rng = SimpleRng::new(42);
    
    // Weighted distribution of common ports (realistic traffic mix)
    let ports = [
        80, 443, 22, 21, 53, 3306, 5432, 6379, 8080, 3000, // 50% common
        22, 80, 443, 53, 3306, 8080,                      // 30% web/DB
        8080, 8443, 9000, 9090,                            // 20% admin/dev
    ];
    let protocols = [6, 6, 6, 17, 17, 6, 6, 6, 6]; // Mostly TCP
    let sizes = [64, 128, 256, 512, 1024, 1400, 64, 128, 256]; // Realistic sizes
    
    // Calculate sleep duration from target PPS
    let sleep_us = 1_000_000 / config.simulation_pps;
    
    loop {
        let src_port: u16 = (rng.next() as u16) % 64512 + 1024;
        let dst_port = ports[rng.range(ports.len())];
        let protocol = protocols[rng.range(protocols.len())];
        let packet_size = sizes[rng.range(sizes.len())] as u32;
        let payload_size = if packet_size > 60 { (packet_size - 60) as u32 } else { 0 };
        
        // Set TCP flags based on port (simulate connection patterns)
        let tcp_flags = match dst_port {
            80 | 443 | 8080 => 0x18, // PSH+ACK (data transfer)
            22 => 0x18,              // SSH also sends data
            21 => 0x02,              // SYN (connection start)
            _ => 0x10,               // ACK (established)
        };
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        
        let features = PacketFeatures {
            src_port,
            dst_port,
            protocol,
            packet_size,
            payload_size,
            tcp_flags,
            timestamp_us: timestamp.as_micros() as u64,
        };
        
        let class_name = classify_port(dst_port);
        let count = counter.fetch_add(1, Ordering::Relaxed);
        
        if config.verbose && count % 1000 == 0 {
            println!("[Simulation] Generated {} packets", count + 1);
        }
        
        let output = ClassifierOutput {
            features,
            class_id: 0,
            class_name: class_name.0,
            confidence: class_name.1,
            flow_stats: None,
        };
        
        if tx.send(output).await.is_err() {
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_micros(sleep_us as u64)).await;
    }
    
    Ok(())
}

// ============================================================================
// ============================================================================
// Pcap Mode Implementation
// ============================================================================

/// Run pcap mode - capture real packets from network interface.
///
/// On Linux: Can use AF_XDP for ~10x performance improvement.
/// On macOS: Uses BPF (limited to 64KB packets).
///
/// Requires the "pcap" feature to be enabled:
///   cargo build --features pcap
///
/// Also requires:
/// - Root permissions (or pcap group on Linux)
/// - Network interface access
async fn start_pcap(
    config: CaptureConfig,
    tx: mpsc::Sender<ClassifierOutput>,
) -> anyhow::Result<()> {
    #[cfg(feature = "pcap")]
    {
        use pcap::{Capture, Device, Mode};
        
        let interface = config.interface.unwrap_or_else(|| "lo0".to_string());
        let filter = config.filter.unwrap_or_else(|| "tcp or udp".to_string());
        
        println!("Opening interface: {}", interface);
        
        let device = Device::from(&interface)?;
        let mut cap = Capture::from_device(device)?
            .mode(Mode::Promiscuous)
            .setnonblock()?;
        
        cap.set_filter(&filter)?;
        
        println!("Capturing packets with filter: {}", filter);
        
        loop {
            match cap.next_packet() {
                Ok(packet) => {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(Duration::ZERO);
                    
                    if let Some(features) = PacketFeatures::from_slice(&packet.data, timestamp) {
                        let class_name = classify_port(features.dst_port);
                        
                        let output = ClassifierOutput {
                            features,
                            class_id: 0,
                            class_name: class_name.0,
                            confidence: class_name.1,
                            flow_stats: None,
                        };
                        
                        if tx.send(output).await.is_err() {
                            break;
                        }
                    }
                }
                Err(pcap::Error::Timeout) => {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                Err(e) => {
                    eprintln!("Packet capture error: {}", e);
                    break;
                }
            }
        }
    }
    
    #[cfg(not(feature = "pcap"))]
    {
        eprintln!("===============================================================");
        eprintln!("PCAP CAPTURE NOT AVAILABLE");
        eprintln!("===============================================================");
        eprintln!("To enable real packet capture:");
        eprintln!("  1. Install libpcap: brew install libpcap (macOS)");
        eprintln!("  2. Rebuild with pcap feature: cargo build --features pcap");
        eprintln!("  3. Run as root or add user to pcap group");
        eprintln!();
        eprintln!("Current: pcap requested, falling back to simulation");
        eprintln!("===============================================================");
        
        let mut sim_config = CaptureConfig::simulation();
        sim_config.simulation_pps = config.simulation_pps;
        sim_config.verbose = config.verbose;
        return start_simulation(sim_config, tx).await;
    }
    
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_port() {
        assert_eq!(classify_port(80), ("HTTP".to_string(), 0.95));
        assert_eq!(classify_port(443), ("HTTPS".to_string(), 0.95));
        assert_eq!(classify_port(22), ("SSH".to_string(), 0.95));
        assert_eq!(classify_port(12345), ("Unknown".to_string(), 0.50));
    }

    #[test]
    fn test_capture_config() {
        let sim = CaptureConfig::simulation();
        assert_eq!(sim.mode, "simulation");
        assert_eq!(sim.simulation_pps, 10000);

        let pcap = CaptureConfig::pcap("eth0");
        assert_eq!(pcap.mode, "pcap");
        assert_eq!(pcap.interface, Some("eth0".to_string()));
    }

    #[test]
    fn test_packet_features_vector() {
        let features = PacketFeatures {
            src_port: 8080,
            dst_port: 443,
            protocol: 6,
            packet_size: 512,
            payload_size: 400,
            tcp_flags: 0x18,
            timestamp_us: 1000,
        };
        
        let vec = features.to_vector();
        assert_eq!(vec.len(), 6);
        assert!(vec.iter().all(|v| *v >= 0.0 && *v <= 1.0));
    }
}