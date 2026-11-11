# Container setup

The checked-in Compose file defines:

| Service | Host port | Purpose |
| --- | --- | --- |
| frontend | 80 | React dashboard served by nginx |
| backend | 8080 | Rust WebSocket statistics server |
| ml-server | 50051 | Python ONNX inference HTTP service |
| mlx-server | 50052 | Optional MLX experiment under the `mlx` profile |

From the repository root:

```sh
docker compose up -d --build
docker compose logs -f
```

Open http://localhost. Stop the stack with `docker compose down`.

## Configuration and limits

The backend loads `/app/config.toml` and uses simulation and port rules by
default. `USE_ML=true` opts into the HTTP inference service at `ML_SERVER_URL`.
The backend checks that service at startup and uses port rules when inference
is unavailable. Per-packet HTTP inference is experimental and will reduce
throughput. The Rust and inference services each expose a `/health` endpoint.

All published host ports bind to loopback. Use an authenticated proxy for
remote access; see the README for origin settings.

The MLX service uses a Linux ARM container definition. Native Apple GPU
acceleration inside that container is not established by this configuration.

Treat these definitions as development setup, not a validated production
deployment. No container or hardware acceptance test is implied by the docs.
