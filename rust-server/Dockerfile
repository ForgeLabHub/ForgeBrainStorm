# Multi-stage build for BrainStorm Rust server.
# Final image is ~15-20 MB (distroless-like).

# ── Builder ───────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
# Create dummy main to cache dependency compilation.
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy real source and build.
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs && cargo build --release

# ── Runtime ───────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash appuser
WORKDIR /app

COPY --from=builder /build/target/release/brainstorm /app/brainstorm
COPY --from=builder /build/migrations /app/migrations
COPY config/ /app/config/

RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=brainstorm=info,tower_http=info
ENV PORT=5000

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

CMD ["/app/brainstorm"]
