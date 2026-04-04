# Network Traffic Classifier

A high-performance real-time network traffic classifier that identifies network protocols (HTTP, HTTPS, SSH, DNS, MySQL, etc.) from packet data. Built with Rust, Python (PyTorch), and React.

![GitHub stars](https://img.shields.io/github/stars/all3n1k/traffic-classifier)
![GitHub license](https://img.shields.io/github/license/all3n1k/traffic-classifier)
![Rust version](https://img.shields.io/badge/Rust-1.94%2B-blue)
![Status](https://img.shields.io/badge/Status-Active-success)

---

## What Does This Do?

```
┌─────────────────────────────────────────────────────────────┐
│                     How It Works                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Network Packet                                            │
│   ┌──────────────┐                                          │
│   │ src: 8080   │                                          │
│   │ dst: 443    │────────┐                                  │
│   │ protocol: 6 │        │                                  │
│   │ size: 512   │        ▼                                  │
│   └──────────────┘    ┌─────────────────┐                  │
│                       │   Classifier    │                  │
│                       │   (Port-based   │                  │
│                       │    or ML model) │                  │
│                       └────────┬────────┘                  │
│                                │                            │
│                       ┌────────▼────────┐                  │
│                       │  HTTPS (95%)    │                  │
│                       │  TCP/443         │                  │
│                       └─────────────────┘                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**In simple terms:** It watches network traffic and guesses what each packet is (web traffic? SSH? Database?) based on the port numbers or a trained ML model.

---

## Quick Start (5 minutes)

### Option A: Docker (Recommended)

```bash
# Start everything with one command
docker compose up -d

# Open dashboard
open http://localhost
```

### Option B: Manual

```bash
# Navigate to project
cd traffic-classifier

# Build backend (first time only)
cd backend
cargo build --release

# Run (simulation mode - creates fake network traffic)
./target/release/traffic-classifier-backend

# In a new terminal - start frontend
cd ../frontend
npm install
npm run dev

# Open http://localhost:5173
```

---

## Command Line Options

```bash
# Default (10,000 packets/second simulation)
./traffic-classifier-backend

# Faster simulation
./traffic-classifier-backend --pps 50000

# Use config file
./traffic-classifier-backend --config config.toml

# Real capture (needs network access)
./traffic-classifier-backend --mode pcap --interface lo0

# Help
./traffic-classifier-backend --help
```

---

## Configuration File

Create a `config.toml` to customize behavior:

```toml
[capture]
mode = "simulation"
simulation_pps = 10000

[server]
port = 8080

[logging]
log_level = "info"
```

Full configuration options are in [`config.toml`](config.toml).

---

## Project Structure

```
traffic-classifier/
├── capture/           # Rust library for packet capture
│   ├── src/capture.rs    # Core capture logic
│   └── src/lib.rs        # Public API
│
├── classifier/        # Python ML model training
│   └── train.py          # PyTorch training script
│
├── backend/           # Rust WebSocket server
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   └── config.rs     # Configuration handling
│   └── config.toml       # Default config
│
├── frontend/          # React dashboard
│   └── src/App.tsx      # Main dashboard UI
│
├── CHANGELOG.md       # Version history
├── README.md          # This file
└── LICENSE            # MIT license
```

---

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| Simulation mode | ✅ Working | Generates realistic synthetic traffic |
| Real pcap capture | 🔄 Planned | Capture from real network interfaces |
| Port-based classification | ✅ Working | Rules-based protocol detection |
| ML classification | 🔄 Planned | PyTorch ONNX model integration |
| Config files | ✅ Working | TOML configuration support |
| WebSocket streaming | ✅ Working | Real-time stats to frontend |
| CLI arguments | ✅ Working | Override config via command line |
| Flow tracking | ✅ Working | Aggregates packets into flows |

### Supported Protocols

- **Web**: HTTP (80), HTTPS (443), HTTP-Alt (8080, 3000)
- **Remote**: SSH (22)
- **File Transfer**: FTP (21)
- **Email**: SMTP (25, 587), POP3 (110), IMAP (143)
- **Database**: MySQL (3306), PostgreSQL (5432), Redis (6379)
- **DNS**: DNS (53)
- **Other**: Unknown (fallback)

---

## Dashboard Preview

The frontend shows:

1. **Stats Cards** - Total packets, packets/second, protocol count, active flows
2. **Protocol Pie Chart** - Distribution of detected protocols
3. **Top Flows Bar Chart** - Most active connections
4. **Throughput Line Chart** - Traffic over time

---

## Development

### Adding Features

1. Make changes to source files
2. Test with `cargo build --release`
3. Update CHANGELOG.md
4. Commit with clear message

### Running Tests

```bash
# Backend tests
cd backend
cargo test

# Capture module tests
cd ../capture
cargo test
```

---

## Docker Deployment

See [DOCKER.md](DOCKER.md) for containerized deployment instructions.

```bash
docker compose up -d
open http://localhost
```

### Features

| Feature | Status | Description |
|---------|--------|-------------|
| Simulation mode | ✅ Working | Generates realistic synthetic traffic |
| Real pcap capture | 🔄 Planned | Capture from real network interfaces |
| Port-based classification | ✅ Working | Rules-based protocol detection |
| ML classification | 🔄 Planned | PyTorch ONNX model integration |
| Config files | ✅ Working | TOML configuration support |
| WebSocket streaming | ✅ Working | Real-time stats to frontend |
| CLI arguments | ✅ Working | Override config via command line |
| Flow tracking | ✅ Working | Aggregates packets into flows |
| Docker support | ✅ Working | Containerized deployment |

---

## Troubleshooting

### "Permission denied" on network interface

**Linux:**
```bash
sudo usermod -a -G pcap $USER
# Log out and back in
```

**macOS:** Use loopback interface (`lo0`) or run without capture (simulation mode).

### Frontend won't connect

- Ensure backend is running: `lsof -i :8080`
- Check firewall: `brew services list` (macOS)

### Build errors

```bash
# Clean and rebuild
cargo clean
cargo build
```

---

## Contributing

1. Fork the repo
2. Create a branch: `git checkout -b feature-name`
3. Commit changes: `git commit -am 'Add feature'`
4. Push: `git push origin main`
5. Open a Pull Request

---

## License

MIT - See [LICENSE](LICENSE) file.

---

## Acknowledgments

- [tokio](https://tokio.rs/) - Async runtime
- [pcap crate](https://crates.io/crates/pcap) - Packet capture
- [PyTorch](https://pytorch.org/) - ML framework
- [Recharts](https://recharts.org/) - Charting library