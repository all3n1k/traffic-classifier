# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **TOML Config Support**: Full configuration file system (`config.toml`)
  - Capture, classification, server, flow, performance, logging, dev sections
  - CLI args: `-c/--config` to load config file
- **Config Module**: Rust config parsing with serde
  - `backend/src/config.rs` - Full configuration handling
  - Default values, file loading, CLI override precedence
- **Structured Logging**: Configurable log levels (trace, debug, info, warn, error)

### Changed
- **README**: Complete rewrite for human readability
  - Quick start in 3 steps
  - ASCII diagram showing how it works
  - Command reference table
  - Troubleshooting section

### Fixed
- **Rust compiler warnings**: Unused imports, duplicate path imports

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