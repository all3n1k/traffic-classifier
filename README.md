# Network Traffic Classifier

A Rust packet-processing prototype with a React dashboard and Python model
experiments. The default backend generates synthetic traffic, labels it using
port-based rules, and streams aggregate statistics over WebSocket.

## Current implementation

| Component | Status |
| --- | --- |
| Simulation and port rules | Implemented in the Rust capture library |
| Dashboard | React charts for protocol counts, throughput, and grouped traffic |
| Configuration | TOML settings and command-line options |
| Live packet capture | Feature-gated pcap code; not part of the default build |
| Model inference | Optional HTTP inference; the default uses port rules |

Protocol labels inferred from ports are heuristics. A service can run on a
nonstandard port, and fixed rule confidence values are not measured accuracy.
Traffic grouped by class and destination port is not full network five-tuple
flow tracking.

## Run the simulation

From the repository root:

```sh
git clone https://github.com/all3n1k/traffic-classifier.git
cd traffic-classifier/backend
cargo run --release
```

In another terminal, start the frontend:

```sh
cd traffic-classifier/frontend
npm install
npm run dev
```

Open http://localhost:5173. The WebSocket backend defaults to port 8080.
Adjust paths if your terminal is already inside the repository.

Backend options include:

```sh
cargo run --release -- --pps 50000
cargo run --release -- --config ../config.toml
cargo run --release -- --help
```

`--pps` controls generated packet rate, not a measured capture-throughput
guarantee. Configuration examples are in [config.toml](config.toml).

## Containers

```sh
docker compose up -d --build
```

The Compose file defines the frontend, backend, and an ONNX server. Published ports bind to loopback. The backend defaults to simulation and port rules; opt into inference with `USE_ML=true`.

See [container notes](DOCKER.md) for the service layout and current limitations.

## Source layout

- `capture/`: packet features, synthetic source, port rules, and pcap scaffolding.
- `backend/`: WebSocket service, configuration, and benchmark recording.
- `frontend/`: React/Vite dashboard.
- `classifier/`: training, capture parsing, ONNX inference, and MLX experiments.
- `traffic-gen/`: Go traffic-generation utility.

## Development

```sh
cd backend
cargo test
cd ../capture
cargo test
```

Build with `cargo test --features pcap` or `cargo run --features pcap -- --mode pcap --interface <interface>`. This requires libpcap; live capture also requires OS capture permission. Requesting pcap in a build without support returns an error instead of simulating traffic.

The WebSocket rejects unrelated browser origins. Keep the dashboard on loopback, or put it behind an authenticated TLS proxy and configure its exact `DASHBOARD_ORIGIN`. Capture statistics can contain sensitive network metadata.

[CHANGELOG.md](CHANGELOG.md) records earlier changes. Issues and pull requests
should include the capture mode, configuration, and reproduction steps.

## License

MIT. See [LICENSE](LICENSE).
