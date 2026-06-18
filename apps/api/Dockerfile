# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.78-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/stellar-iot-api /usr/local/bin/stellar-iot-api

EXPOSE 8000

ENV RUST_LOG=info

CMD ["stellar-iot-api"]
