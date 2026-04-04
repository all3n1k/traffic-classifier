# Traffic Classifier Docker Deployment

Containerized deployment for the traffic classifier using Docker Compose.

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Frontend    │────▶│   Backend   │────▶│  ML Server  │
│  (nginx)     │◀────│   (Rust)    │◀────│  (Python)  │
│   :80        │     │   :8080     │     │   :50051    │
└─────────────┘     └─────────────┘     └─────────────┘
```

## Quick Start

```bash
# Build and start all services
make docker-build
make docker-up

# Open browser
open http://localhost
```

Or use Docker Compose directly:

```bash
docker compose up -d
docker compose logs -f
```

## Services

| Service | Port | Description |
|---------|------|-------------|
| frontend | 80 | React dashboard (nginx) |
| backend | 8080 | Rust WebSocket server |
| ml-server | 50051 | Python ONNX inference |

## Configuration

Edit `docker-config.toml` to customize settings:

```toml
[capture]
mode = "simulation"        # or "pcap"
simulation_pps = 10000

[classification]
use_ml = false             # set to true to enable ML
```

## With ML Classification

To enable ML-based classification:

```bash
# Edit docker-config.toml
[classification]
use_ml = true

# Restart
docker compose restart backend
```

## Makefile Commands

```bash
make docker-build    # Build all Docker images
make docker-up       # Start all containers
make docker-down     # Stop all containers
make docker-logs     # Tail all logs
make docker-restart  # Restart everything
make docker-shell-backend  # Shell into backend
make docker-shell-ml      # Shell into ML server
make docker-clean    # Remove containers and images
```

## Local Development

Build without cache:

```bash
docker compose build --no-cache
docker compose up -d
```

## Networking

All services communicate via internal Docker network `traffic-net`.

- Backend connects to ML server at `http://ml-server:50051`
- Frontend proxies WebSocket to `http://backend:8080`

## Production Notes

For production deployment:
1. Enable TLS on nginx
2. Set proper `server.host = "0.0.0.0"` in config
3. Consider using Docker secrets for sensitive config
4. Add resource limits in docker-compose.yml
