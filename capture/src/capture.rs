use std::time::Duration;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketFeatures {
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub packet_size: u32,
    pub payload_size: u32,
    pub tcp_flags: u8,
    pub timestamp_us: u64,
}

impl PacketFeatures {
    pub fn from_slice(data: &[u8], timestamp: Duration) -> Option<Self> {
        if data.len() < 34 { return None; }
        let eth_type = u16::from_be_bytes([data[12], data[13]]);
        if eth_type != 0x0800 { return None; }
        let ip_header_len = ((data[14] & 0x0F) as usize) * 4;
        if data.len() < 14 + ip_header_len + 4 { return None; }
        let protocol = data[14 + 9];
        let (src_port, dst_port, payload_size, tcp_flags) = match protocol {
            6 => {
                let ip_start = 14 + ip_header_len;
                if data.len() < ip_start + 20 { return None; }
                let src_p = u16::from_be_bytes([data[ip_start], data[ip_start+1]]);
                let dst_p = u16::from_be_bytes([data[ip_start+2], data[ip_start+3]]);
                (src_p, dst_p, (data.len() - ip_start - 20) as u32, data[ip_start + 13])
            }
            17 => {
                let ip_start = 14 + ip_header_len;
                if data.len() < ip_start + 8 { return None; }
                let src_p = u16::from_be_bytes([data[ip_start], data[ip_start+1]]);
                let dst_p = u16::from_be_bytes([data[ip_start+2], data[ip_start+3]]);
                (src_p, dst_p, (data.len() - ip_start - 8) as u32, 0)
            }
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierOutput {
    pub features: PacketFeatures,
    pub class_id: u8,
    pub class_name: String,
    pub confidence: f32,
    pub flow_stats: Option<FlowSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSummary {
    pub dst_port: u16,
    pub protocol: String,
    pub class_name: String,
    pub packet_count: u64,
    pub byte_count: u64,
}

struct SimpleRng { state: u64 }

impl SimpleRng {
    fn new(seed: u64) -> Self { Self { state: seed } }
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state >> 16
    }
    fn range(&mut self, max: usize) -> usize { (self.next() as usize) % max }
}

pub async fn start_capture(
    _device: String,
    tx: mpsc::Sender<ClassifierOutput>,
) -> anyhow::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    let counter = Arc::new(AtomicU64::new(0));
    let mut rng = SimpleRng::new(42);
    
    let ports = [80, 443, 22, 21, 53, 3306, 5432, 6379, 8080, 3000, 22, 80, 443, 53, 3306, 8080];
    let protocols = [6, 6, 6, 17, 17, 6, 6, 6, 6]; 
    let sizes = [64, 128, 256, 512, 1024, 1400, 64, 128, 256];
    
    loop {
        let src_port: u16 = (rng.next() as u16) % 64512 + 1024;
        let dst_port = ports[rng.range(ports.len())];
        let protocol = protocols[rng.range(protocols.len())];
        let packet_size = sizes[rng.range(sizes.len())] as u32;
        let payload_size = if packet_size > 60 { (packet_size - 60) as u32 } else { 0 };
        let tcp_flags = match dst_port {
            80 | 443 | 8080 => 0x18,
            22 => 0x18,
            21 => 0x02,
            _ => 0x10,
        };
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        
        let features = PacketFeatures { src_port, dst_port, protocol, packet_size, payload_size, tcp_flags, timestamp_us: timestamp.as_micros() as u64 };
        let class_name = classify_port(dst_port);
        let count = counter.fetch_add(1, Ordering::Relaxed);
        
        let output = ClassifierOutput { features, class_id: 0, class_name: class_name.0, confidence: class_name.1, flow_stats: None };
        
        if tx.send(output).await.is_err() { break; }
        if count % 1000 == 0 { println!("Simulated {} packets", count + 1); }
        tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
    }
    Ok(())
}

fn classify_port(port: u16) -> (String, f32) {
    match port {
        80 => ("HTTP".to_string(), 0.95),
        443 => ("HTTPS".to_string(), 0.95),
        22 => ("SSH".to_string(), 0.95),
        21 => ("FTP".to_string(), 0.90),
        53 => ("DNS".to_string(), 0.90),
        25 | 587 | 465 => ("SMTP".to_string(), 0.85),
        3306 => ("MySQL".to_string(), 0.90),
        5432 => ("PostgreSQL".to_string(), 0.90),
        6379 => ("Redis".to_string(), 0.90),
        8080 | 8000 | 3000 => ("HTTP-Alt".to_string(), 0.80),
        _ => ("Unknown".to_string(), 0.50),
    }
}