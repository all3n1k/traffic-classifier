FROM rust:1.94-bookworm AS builder
WORKDIR /workspace
COPY capture ./capture
COPY backend ./backend
RUN cargo build --locked --release --manifest-path backend/Cargo.toml

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/backend/target/release/traffic-classifier-backend /app/backend
COPY config.toml /app/config.toml
ENV SERVER_HOST=0.0.0.0
EXPOSE 8080
CMD ["/app/backend", "--config", "/app/config.toml"]
