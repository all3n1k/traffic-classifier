# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Project Initialization**: Created traffic classifier with Rust + PyTorch + React stack
- **Simulation Mode**: Implemented synthetic packet generation (~10K pkt/s) for testing without real network access
- **WebSocket Backend**: Real-time stats streaming at 5Hz to frontend
- **React Dashboard**: Live visualization with protocol distribution, top flows, throughput charts
- **GitHub Repository**: Public repo with comprehensive README and MIT license
- **Clean Git History**: Removed build artifacts and node_modules from version control
- **CaptureConfig**: Unified configuration system for simulation/pcap modes
- **CLI Arguments**: Command-line options for mode, interface, PPS, port, verbose
- **Extensive Documentation**: Full module-level docs in Rust code

### Changed
- **capture/capture.rs**: Refactored with CaptureConfig, proper documentation, test coverage
- **backend/main.rs**: Added CLI argument parsing, configurable WebSocket port

### Architecture
```
Packet Source (sim/real) → Rust Capture → ML Classifier → WebSocket Server → React Dashboard
```

### Components
- `capture/` - Rust packet capture library (simulation mode)
- `classifier/` - PyTorch training pipeline (ready for ONNX export)
- `backend/` - WebSocket server with tokio
- `frontend/` - React + TypeScript dashboard with Recharts

## [0.1.0] - 2024-04-02

### Added
- Real-time packet classification (HTTP, HTTPS, SSH, FTP, DNS, MySQL, PostgreSQL, Redis)
- Flow-based statistics tracking
- Dark theme dashboard (GitHub-inspired)
- Performance metrics (packets/sec, total packets, active flows)
- Protocol distribution pie chart
- Top flows bar chart
- Throughput over time line chart

### Technical Details
- **Capture**: Uses custom SimpleRng for thread-safe simulation (not `rand::thread_rng` which is `!Send`)
- **WebSocket**: Broadcasts stats every 200ms to all connected clients
- **State Management**: AtomicU64 for counters, RwLock for collections
- **Frontend**: React 18 + TypeScript + Vite + Recharts

### Known Limitations
- Simulation mode only (no real packet capture)
- ML model not yet integrated (ONNX runtime pending)
- No configuration file (hardcoded values)

---

## Planned for 0.2.0

### High Priority
- [ ] Real packet capture with pcap crate
- [ ] AF_XDP support for Linux (10x faster than pcap)
- [ ] ONNX model integration for ML-based classification

### Medium Priority
- [ ] Configuration file (config.toml)
- [ ] Benchmark utilities
- [ ] Prometheus metrics endpoint

### Lower Priority
- [ ] Docker Compose deployment
- [ ] Kubernetes manifests
- [ ] Anomaly detection

---

## How to Document Changes

When making changes, add a new entry following this format:

```markdown
### Changed
- **component**: Brief description of what changed and why
```

### Change Types
- **Added** - New feature
- **Changed** - Existing functionality modified
- **Deprecated** - Soon-to-be removed feature
- **Removed** - Feature removed
- **Fixed** - Bug fix
- **Security** - Security-related change

---

## Version History

- [0.1.0](#010---2024-04-02) - Initial release with simulation mode