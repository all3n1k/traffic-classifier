[package]
name = "traffic-classifier-backend"
version = "0.2.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
capture = { path = "../capture" }
chrono = "0.4"
toml = "0.8"
reqwest = { version = "0.12", features = ["json"] }

[features]
default = ["pcap"]
pcap = ["capture/pcap"]

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
