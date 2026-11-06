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

The backend service mounts `config.toml` at `/app/config.toml`. Its declared
environment selects simulation, a generated rate of 10,000 packets/second, and
`USE_ML=false`. Check the backend's actual configuration loading when changing
these values; a declared environment variable alone does not prove a code path
uses it.

The default packet-processing task uses port rules. Although the repository
contains `MLClient` and the Python inference service, the backend does not
instantiate that client in its capture task. Starting `ml-server` or changing
`use_ml` is not evidence of model-backed end-to-end classification.

The backend Compose healthcheck requests HTTP `/health` on port 8080, while
the current Rust listener accepts WebSocket connections. This mismatch can mark
a running backend unhealthy. The inference server's separate `/health`
endpoint is implemented.

The MLX service uses a Linux ARM container definition. Native Apple GPU
acceleration inside that container is not established by this configuration.

Treat these definitions as development setup, not a validated production
deployment. No container or hardware acceptance test is implied by the docs.
