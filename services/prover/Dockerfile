# ============================================================
# Prover service (Rust + axum + Sunspot CLI wrapper)
# ============================================================
# NOTE: This service requires `sunspot` CLI and circuit artifacts
# (withdrawal.json, withdrawal.ccs, withdrawal.pk) to be available.
# It is intended to run inside the `solana` container, not standalone.

FROM rust:1.89-slim AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
  libssl-dev pkg-config build-essential \
  && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
  libssl3 ca-certificates curl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/zkpool_prover /app/zkpool-prover

RUN useradd -u 1001 -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 4002

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:4002/health || exit 1

CMD ["/app/zkpool-prover"]
