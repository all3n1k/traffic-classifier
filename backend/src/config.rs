//! Configuration management for traffic classifier.
//! 
//! Handles loading configuration from TOML files and command-line arguments.
//! Command-line arguments take precedence over config file values.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Capture configuration
    #[serde(default)]
    pub capture: CaptureSettings,
    
    /// Classification configuration  
    #[serde(default)]
    pub classification: ClassificationConfig,
    
    /// WebSocket server configuration
    #[serde(default)]
    pub server: ServerConfig,
    
    /// Flow tracking configuration
    #[serde(default)]
    pub flow: FlowConfig,
    
    /// Performance configuration
    #[serde(default)]
    pub performance: PerformanceConfig,
    
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
    
    /// Development settings
    #[serde(default)]
    pub dev: DevConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            capture: CaptureSettings::default(),
            classification: ClassificationConfig::default(),
            server: ServerConfig::default(),
            flow: FlowConfig::default(),
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
            dev: DevConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from TOML file.
    /// 
    /// # Arguments
    /// * `path` - Path to config file
    /// 
    /// # Errors
    /// Returns error if file cannot be read or parsed
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Create config with CLI overrides.
    /// 
    /// Loads from file first, then applies CLI overrides.
    pub fn from_cli(
        config_file: Option<&str>,
        mode: Option<&str>,
        interface: Option<&str>,
        pps: Option<u32>,
        port: Option<u16>,
        verbose: Option<bool>,
    ) -> anyhow::Result<Self> {
        // Start with defaults
        let mut config = Config::default();
        
        // Load from file if provided
        if let Some(path) = config_file {
            let file_config = Config::from_file(&PathBuf::from(path))?;
            config = file_config;
        }
        
        // Apply CLI overrides
        if let Some(m) = mode {
            config.capture.mode = m.to_string();
        }
        if let Some(i) = interface {
            config.capture.interface = Some(i.to_string());
        }
        if let Some(p) = pps {
            config.capture.simulation_pps = p;
        }
        if let Some(p) = port {
            config.server.port = p;
        }
        if let Some(v) = verbose {
            config.capture.verbose = v;
        }
        
        Ok(config)
    }
}

// ============================================================================
// Sub-configurations
// ============================================================================

/// Backend capture configuration (renamed to avoid conflict with capture crate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSettings {
    /// Capture mode: "simulation" or "pcap"
    #[serde(default = "default_mode")]
    pub mode: String,
    
    /// Network interface for pcap mode
    #[serde(default)]
    pub interface: Option<String>,
    
    /// BPF filter for pcap mode
    #[serde(default)]
    pub filter: Option<String>,
    
    /// Packets per second for simulation mode
    #[serde(default = "default_pps")]
    pub simulation_pps: u32,
    
    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,
    
    /// Maximum packets per second (0 = unlimited)
    #[serde(default)]
    pub max_pps: u32,
}

fn default_mode() -> String { "simulation".to_string() }
fn default_pps() -> u32 { 10000 }

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            mode: "simulation".to_string(),
            interface: None,
            filter: None,
            simulation_pps: 10000,
            verbose: false,
            max_pps: 0,
        }
    }
}

impl From<CaptureSettings> for capture::CaptureConfig {
    fn from(cfg: CaptureSettings) -> Self {
        let mut capture_cfg = capture::CaptureConfig::simulation();
        capture_cfg.mode = cfg.mode;
        capture_cfg.interface = cfg.interface;
        capture_cfg.filter = cfg.filter;
        capture_cfg.simulation_pps = cfg.simulation_pps;
        capture_cfg.verbose = cfg.verbose;
        capture_cfg
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationConfig {
    /// Use ML model instead of rule-based classification
    #[serde(default)]
    pub use_ml: bool,
    
    /// Path to ONNX model file
    #[serde(default)]
    pub model_path: String,
    
    /// Confidence threshold for classification
    #[serde(default = "default_confidence")]
    pub confidence_threshold: f32,
    
    /// Unknown classification fallback
    #[serde(default = "default_unknown")]
    pub unknown_class: String,
}

fn default_confidence() -> f32 { 0.5 }
fn default_unknown() -> String { "Unknown".to_string() }

impl Default for ClassificationConfig {
    fn default() -> Self {
        Self {
            use_ml: false,
            model_path: "classifier/TrafficClassifier.onnx".to_string(),
            confidence_threshold: 0.5,
            unknown_class: "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// WebSocket server host
    #[serde(default = "default_host")]
    pub host: String,
    
    /// WebSocket server port
    #[serde(default = "default_server_port")]
    pub port: u16,
    
    /// Maximum concurrent WebSocket connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    
    /// Stats update interval in milliseconds
    #[serde(default = "default_stats_interval")]
    pub stats_interval_ms: u32,
    
    /// Enable ping/pong heartbeats
    #[serde(default = "default_true")]
    pub enable_heartbeat: bool,
    
    /// Heartbeat interval in seconds
    #[serde(default = "default_heartbeat")]
    pub heartbeat_interval: u32,
}

fn default_host() -> String { "127.0.0.1".to_string() }
fn default_server_port() -> u16 { 8080 }
fn default_max_connections() -> u32 { 100 }
fn default_stats_interval() -> u32 { 200 }
fn default_true() -> bool { true }
fn default_heartbeat() -> u32 { 30 }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            stats_interval_ms: 200,
            enable_heartbeat: true,
            heartbeat_interval: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConfig {
    /// Enable flow tracking
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Flow timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    
    /// Maximum number of concurrent flows
    #[serde(default = "default_max_flows")]
    pub max_flows: u32,
    
    /// Aggregate packets into micro-batches
    #[serde(default = "default_batch")]
    pub batch_size: u32,
}

fn default_timeout() -> u32 { 300 }
fn default_max_flows() -> u32 { 10000 }
fn default_batch() -> u32 { 100 }

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 300,
            max_flows: 10000,
            batch_size: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Worker threads (0 = auto)
    #[serde(default)]
    pub worker_threads: u32,
    
    /// Channel buffer size
    #[serde(default = "default_buffer")]
    pub channel_buffer_size: u32,
    
    /// Enable metrics collection
    #[serde(default = "default_true")]
    pub metrics_enabled: bool,
    
    /// Metrics export interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub metrics_interval: u32,
}

fn default_buffer() -> u32 { 1000 }
fn default_metrics_interval() -> u32 { 60 }

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: 0,
            channel_buffer_size: 1000,
            metrics_enabled: true,
            metrics_interval: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Log to file
    #[serde(default)]
    pub log_file: Option<String>,
    
    /// Enable JSON logging
    #[serde(default)]
    pub json_log: bool,
}

fn default_log_level() -> String { "info".to_string() }

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_file: None,
            json_log: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    /// Simulate packet loss percentage
    #[serde(default)]
    pub packet_loss_percent: u32,
    
    /// Artificial delay in microseconds
    #[serde(default)]
    pub artificial_delay_us: u32,
    
    /// Simulation seed
    #[serde(default = "default_seed")]
    pub simulation_seed: u64,
}

fn default_seed() -> u64 { 42 }

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            packet_loss_percent: 0,
            artificial_delay_us: 0,
            simulation_seed: 42,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.capture.mode, "simulation");
        assert_eq!(config.capture.simulation_pps, 10000);
        assert_eq!(config.server.port, 8080);
    }
    
    #[test]
    fn test_cli_override() {
        let config = Config::from_cli(
            None,                         // no config file
            Some("pcap"),                 // mode override
            Some("eth0"),                // interface
            Some(50000),                 // pps
            Some(9000),                  // port
            Some(true),                  // verbose
        ).unwrap();
        
        assert_eq!(config.capture.mode, "pcap");
        assert_eq!(config.capture.interface, Some("eth0".to_string()));
        assert_eq!(config.capture.simulation_pps, 50000);
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.capture.verbose, true);
    }
}