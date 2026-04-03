use anyhow::Result;
use capture::{start_capture, CaptureConfig, ClassifierOutput};
use config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;

mod config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsMessage {
    pub total_packets: u64,
    pub packets_per_second: f64,
    pub classifications: HashMap<String, u64>,
    pub flows: Vec<FlowSummary>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSummary {
    pub dst_port: u16,
    pub protocol: String,
    pub class_name: String,
    pub packet_count: u64,
    pub byte_count: u64,
}

pub struct AppState {
    total_packets: Arc<AtomicU64>,
    classification_counts: RwLock<HashMap<String, u64>>,
    last_reset: std::time::Instant,
    packets_this_second: Arc<AtomicU64>,
    current_pps: RwLock<f64>,
    flows: Arc<RwLock<HashMap<String, FlowSummary>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            total_packets: Arc::new(AtomicU64::new(0)),
            classification_counts: RwLock::new(HashMap::new()),
            last_reset: std::time::Instant::now(),
            packets_this_second: Arc::new(AtomicU64::new(0)),
            current_pps: RwLock::new(0.0),
            flows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn record_packet(&self, output: &ClassifierOutput) {
        self.total_packets.fetch_add(1, Ordering::Relaxed);
        self.packets_this_second.fetch_add(1, Ordering::Relaxed);

        let mut counts = self.classification_counts.write().await;
        *counts.entry(output.class_name.clone()).or_insert(0) += 1;

        let flow_key = format!("{}:{}", output.class_name, output.features.dst_port);
        let mut flows = self.flows.write().await;
        let flow = flows.entry(flow_key.clone()).or_insert(FlowSummary {
            dst_port: output.features.dst_port,
            protocol: match output.features.protocol {
                6 => "TCP".to_string(),
                17 => "UDP".to_string(),
                _ => "Other".to_string(),
            },
            class_name: output.class_name.clone(),
            packet_count: 0,
            byte_count: 0,
        });
        flow.packet_count += 1;
        flow.byte_count += output.features.packet_size as u64;
    }

    async fn get_stats(&self) -> StatsMessage {
        let elapsed = self.last_reset.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            let pps = self.packets_this_second.swap(0, Ordering::Relaxed) as f64 / elapsed;
            *self.current_pps.write().await = pps;
        }

        let flows_list = {
            let flows = self.flows.read().await;
            flows.values().cloned().collect()
        };

        StatsMessage {
            total_packets: self.total_packets.load(Ordering::Relaxed),
            packets_per_second: *self.current_pps.read().await,
            classifications: self.classification_counts.read().await.clone(),
            flows: flows_list,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

async fn handle_ws_connection(
    stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    state: Arc<AppState>,
    stats_interval_ms: u32,
) {
    let (mut write, mut read) = stream.split();

    let state_clone = state.clone();
    let interval_duration = std::time::Duration::from_millis(stats_interval_ms as u64);
    let mut interval = tokio::time::interval(interval_duration);

    loop {
        tokio::select! {
            msg = read.next() => {
                if let Some(Ok(Message::Text(text))) = msg {
                    if text == "ping" {
                        let _ = write.send(Message::Text("pong".to_string())).await;
                    }
                } else if msg.is_none() {
                    break;
                }
            }
            _ = interval.tick() => {
                let stats = state_clone.get_stats().await;
                if let Ok(json) = serde_json::to_string(&stats) {
                    let _ = write.send(Message::Text(json)).await;
                }
            }
        }
    }
}

async fn start_capture_task(
    config: CaptureConfig,
    state: Arc<AppState>,
    channel_buffer_size: u32,
) {
    let (tx, mut rx) = mpsc::channel::<ClassifierOutput>(channel_buffer_size as usize);

    tokio::spawn(async move {
        if let Err(e) = start_capture(config, tx).await {
            eprintln!("Capture error: {}", e);
        }
    });

    while let Some(output) = rx.recv().await {
        state.record_packet(&output).await;
    }
}

fn print_help() {
    println!("Traffic Classifier Backend v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("  traffic-classifier-backend [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -c, --config FILE      Load configuration from TOML file");
    println!("  -m, --mode MODE        Capture mode: simulation (default) or pcap");
    println!("  -i, --interface IFACE  Network interface for pcap mode (e.g., eth0, lo0)");
    println!("  -p, --pps RATE         Packets per second for simulation (default: 10000)");
    println!("      --port PORT        WebSocket server port (default: 8080)");
    println!("  -v, --verbose          Enable verbose logging");
    println!("  -h, --help             Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  # Default: simulation mode, 10K pps, port 8080");
    println!("  traffic-classifier-backend");
    println!();
    println!("  # High-speed simulation");
    println!("  traffic-classifier-backend --pps 50000");
    println!();
    println!("  # Load from config file");
    println!("  traffic-classifier-backend --config config.toml");
    println!();
    println!("  # Real capture on loopback");
    println!("  traffic-classifier-backend --mode pcap --interface lo0");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let mut config_file: Option<&str> = None;
    let mut mode: Option<&str> = None;
    let mut interface: Option<&str> = None;
    let mut pps: Option<u32> = None;
    let mut port: Option<u16> = None;
    let mut verbose: Option<bool> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--config" if i + 1 < args.len() => {
                config_file = Some(&args[i + 1]);
                i += 2;
            }
            "-m" | "--mode" if i + 1 < args.len() => {
                mode = Some(&args[i + 1]);
                i += 2;
            }
            "-i" | "--interface" if i + 1 < args.len() => {
                interface = Some(&args[i + 1]);
                i += 2;
            }
            "-p" | "--pps" if i + 1 < args.len() => {
                pps = Some(args[i + 1].parse().unwrap_or(10000));
                i += 2;
            }
            "--port" if i + 1 < args.len() => {
                port = Some(args[i + 1].parse().unwrap_or(8080));
                i += 2;
            }
            "-v" | "--verbose" => {
                verbose = Some(true);
                i += 1;
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => i += 1,
        }
    }

    // Load configuration
    let config = Config::from_cli(
        config_file,
        mode,
        interface,
        pps,
        port,
        verbose,
    )?;

    // Setup logging based on config
    // Use RUST_LOG environment variable for filtering
    if let Ok(_) = std::env::var("RUST_LOG") {
        tracing_subscriber::fmt::init();
    } else {
        // Set default log level based on config
        let log_level = match config.logging.log_level.as_str() {
            "trace" => "trace",
            "debug" => "debug",
            "info" => "info",
            "warn" => "warn",
            "error" => "error",
            _ => "info",
        };
        std::env::set_var("RUST_LOG", log_level);
        let _ = tracing_subscriber::fmt::try_init();
    }

    println!("Starting Traffic Classifier Backend v{}", env!("CARGO_PKG_VERSION"));
    println!("  Capture mode: {}", config.capture.mode);
    if let Some(ref iface) = config.capture.interface {
        println!("  Interface: {}", iface);
    }
    if config.capture.mode == "simulation" {
        println!("  Target PPS: {}", config.capture.simulation_pps);
    }
    println!("WebSocket server: ws://{}:{}", config.server.host, config.server.port);

    let state = Arc::new(AppState::new());
    let state_clone = state.clone();

    // Start capture task with config
    let capture_config: capture::CaptureConfig = config.capture.clone().into();
    let capture_state = state.clone();
    let buffer_size = config.performance.channel_buffer_size;
    
    tokio::spawn(async move {
        start_capture_task(capture_config, capture_state, buffer_size).await;
    });

    // Start WebSocket server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let stats_interval = config.server.stats_interval_ms;

    println!("Server listening on {}", addr);
    println!("Stats updates every {}ms", stats_interval);

    loop {
        if let Ok((stream, _addr)) = listener.accept().await {
            let state = state_clone.clone();
            let interval = stats_interval;
            tokio::spawn(async move {
                if let Ok(ws_stream) = accept_async(stream).await {
                    handle_ws_connection(ws_stream, state, interval).await;
                }
            });
        }
    }
}