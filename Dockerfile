FROM rust:1-slim-trixie AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release

FROM debian:trixie-slim
COPY --from=builder /app/target/release/deluge-retain /usr/local/bin/deluge-retain
USER 1000
ENTRYPOINT ["deluge-retain"]
