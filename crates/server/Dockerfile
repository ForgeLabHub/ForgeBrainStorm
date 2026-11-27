# Multi-stage build for ForgeBrain server.
# Final image is ~15-20 MB (distroless-like).

# ── Builder ───────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

WORKDIR /build

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./
COPY crates/server/Cargo.toml crates/server/

# Create dummy main to cache dependency compilation
RUN mkdir -p crates/server/src && echo "fn main() {}" > crates/server/src/main.rs
RUN cargo build --release --package forgebrain-server && rm -rf crates/server/src

# Copy real source and build
COPY crates/server/src crates/server/src
COPY crates/server/migrations crates/server/migrations
COPY crates/server/config crates/server/config
RUN touch crates/server/src/main.rs && cargo build --release --package forgebrain-server

# ── Runtime ───────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash appuser
WORKDIR /app

COPY --from=builder /build/target/release/forgebrain-server /app/forgebrain-server
COPY --from=builder /build/crates/server/migrations /app/migrations
COPY crates/server/config/ /app/config/

RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=forgebrain_server=info,tower_http=info
ENV PORT=5000

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

CMD ["/app/forgebrain-server"]
