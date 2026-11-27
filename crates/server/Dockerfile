# Multi-stage build for the BrainStorm Rust server.
#
# Build from the repository root — the crate is a member of a Cargo workspace:
#   docker build -f crates/server/Dockerfile -t brainstorm .
# Final image is ~15-20 MB (distroless-like).

# ── Builder ───────────────────────────────────────────────────
# Keep in sync with rust-toolchain.toml (CI fails the build if they drift).
FROM rust:1.89-slim AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY xtask/Cargo.toml xtask/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml

# Compile the dependency graph once against stub sources, so edits to the real
# code do not invalidate ~400 crates.
RUN mkdir -p crates/server/src xtask/src \
    && echo "" > crates/server/src/lib.rs \
    && echo "fn main() {}" > crates/server/src/main.rs \
    && echo "fn main() {}" > xtask/src/main.rs \
    && cargo build --release -p brainstorm \
    && rm -rf crates/server/src

# config/default.toml is embedded at compile time by config.rs.
COPY crates/server/src/ crates/server/src/
COPY crates/server/config/ crates/server/config/
COPY crates/server/migrations/ crates/server/migrations/
RUN touch crates/server/src/main.rs && cargo build --release -p brainstorm

# ── Runtime ───────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash appuser
WORKDIR /app

COPY --from=builder /build/target/release/brainstorm /app/brainstorm
COPY --from=builder /build/crates/server/migrations /app/migrations
COPY crates/server/config/ /app/config/

RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=brainstorm=info,tower_http=info
ENV PORT=5000

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

CMD ["/app/brainstorm"]
