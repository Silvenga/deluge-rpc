# Stage 1: Build
FROM rust:1.85-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

# Stage 2: Runtime
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/deluge-retain /usr/local/bin/deluge-retain
USER 65534
ENTRYPOINT ["deluge-retain"]
