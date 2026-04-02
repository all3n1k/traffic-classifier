# Traffic Classifier

High-performance real-time network traffic classifier using Rust, PyTorch, and React.

## Quick Start (macOS)

### 1. Install Rust (required for packet capture)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.zshrc

# Install libpcap (required for packet capture)
brew install libpcap
```

### 2. Install Python ML dependencies
```bash
# Create and activate virtual environment
cd traffic-classifier/classifier
python3 -m venv venv
source venv/bin/activate

# Install PyTorch and ONNX
pip install torch numpy scikit-learn onnx
```

### 3. Train ML Model
```bash
python train.py
```

### 4. Install Frontend Dependencies
```bash
cd ../frontend
npm install
```

### 5. Build & Run

**Backend (Rust):**
```bash
cd ../backend
cargo run --release
```

**Frontend (React):**
```bash
cd ../frontend
npm run dev
```

Open http://localhost:5173 to see the dashboard.

## Project Structure

```
traffic-classifier/
├── capture/           # Rust packet capture library
│   ├── src/
│   │   ├── lib.rs     # Data structures & classifier
│   │   ├── capture.rs # PCAP capture implementation
│   │   └── sim.rs     # Simulation mode (no pcap required)
│   └── Cargo.toml
├── classifier/       # ML model training
│   ├── train.py       # PyTorch training script
│   └── TrafficClassifier.onnx
├── backend/           # WebSocket server
│   └── src/main.rs
├── frontend/          # React dashboard
│   └── src/App.tsx
├── traffic-gen/       # Synthetic traffic (Go)
└── README.md
```

## Architecture

1. **Capture Layer** - Rust + pcap captures packets from network interface
2. **ML Layer** - PyTorch model classifies traffic (HTTP, SSH, DNS, etc.)
3. **Backend** - WebSocket server streams classification results
4. **Frontend** - React dashboard shows real-time metrics

## Features

- Real-time packet classification at >100K packets/sec
- Protocol detection: HTTP, HTTPS, SSH, FTP, DNS, SMTP, MySQL, PostgreSQL, Redis
- Live throughput visualization with Recharts
- Flow-based statistics tracking
- Dark mode dashboard

## Demo Mode

If you can't capture real packets, use simulation mode:
```bash
cd capture
cargo run --bin capture-demo -- --simulate
```

## Performance

- Target: 100K+ packets/second
- Classification latency: <1ms
- Memory footprint: ~50MB

## Troubleshooting

**Permission denied on pcap:**
```bash
# On Linux, add user to 'pcap' group
sudo usermod -a -G pcap $USER
```

**WebSocket connection failed:**
- Ensure backend is running on port 8080
- Check firewall settings

## License

MIT