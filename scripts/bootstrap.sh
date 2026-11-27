#!/usr/bin/env bash
# Idempotent recovery for the local dev stack: PostgreSQL, Rust backend, SvelteKit.
# Safe to run on a fresh container — installs whatever is missing, then starts everything.
#
# Usage: scripts/bootstrap.sh [--build] [--seed]
#   --build  force a cargo rebuild
#   --seed   create demo users and play a few scripted games
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_DIR="$ROOT/.run"
BACKEND_PORT="${PORT:-5000}"
FRONTEND_PORT="${FRONTEND_PORT:-12000}"
FORCE_BUILD=0
SEED=0

for arg in "$@"; do
	case "$arg" in
		--build) FORCE_BUILD=1 ;;
		--seed) SEED=1 ;;
	esac
done

mkdir -p "$RUN_DIR"
log() { printf '[bootstrap] %s\n' "$*"; }

# ── PostgreSQL ────────────────────────────────────────────────
if ! command -v pg_isready >/dev/null 2>&1; then
	log "installing PostgreSQL"
	sudo DEBIAN_FRONTEND=noninteractive apt-get update -qq
	sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq postgresql postgresql-contrib
fi

if ! pg_isready -q -h 127.0.0.1 -p 5432; then
	log "starting PostgreSQL"
	sudo service postgresql start >/dev/null 2>&1
	for _ in $(seq 1 20); do
		pg_isready -q -h 127.0.0.1 -p 5432 && break
		if [ "$_" = 3 ]; then
			sudo pg_ctlcluster "$(ls /etc/postgresql 2>/dev/null | head -1)" main start >/dev/null 2>&1
		fi
		sleep 1
	done
fi

if ! pg_isready -q -h 127.0.0.1 -p 5432; then
	log "postgres: FAILED to start"
	exit 1
fi
log "postgres: ok"

if ! sudo -u postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='brainstorm'" | grep -q 1; then
	log "creating role brainstorm"
	sudo -u postgres psql -q -c "CREATE ROLE brainstorm LOGIN PASSWORD 'brainstorm' CREATEDB"
fi
if ! sudo -u postgres psql -tAc "SELECT 1 FROM pg_database WHERE datname='brainstorm'" | grep -q 1; then
	log "creating database brainstorm"
	sudo -u postgres createdb -O brainstorm brainstorm
fi

# ── Rust toolchain ────────────────────────────────────────────
if ! command -v cargo >/dev/null 2>&1 && [ -x "$HOME/.cargo/bin/cargo" ]; then
	export PATH="$HOME/.cargo/bin:$PATH"
fi
if ! command -v cargo >/dev/null 2>&1; then
	log "installing Rust toolchain (rustup, minimal profile)"
	curl -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
	sh /tmp/rustup-init.sh -y --profile minimal --default-toolchain stable --no-modify-path
	export PATH="$HOME/.cargo/bin:$PATH"
fi

# ── Backend build ─────────────────────────────────────────────
BIN="$ROOT/target/debug/brainstorm"
if [ "$FORCE_BUILD" = 1 ] || [ ! -x "$BIN" ] ||
	[ -n "$(find "$ROOT/crates/server/src" "$ROOT/crates/server/migrations" -newer "$BIN" -print -quit 2>/dev/null)" ]; then
	log "building backend"
	(cd "$ROOT" && cargo build --quiet -p brainstorm) || {
		log "backend build FAILED"
		exit 1
	}
fi

# ── Services ──────────────────────────────────────────────────
PORT="$BACKEND_PORT" "$ROOT/scripts/dev-up.sh" "$FRONTEND_PORT" | sed 's/^/[bootstrap] /'

for _ in $(seq 1 30); do
	if curl -fsS -m 2 "http://127.0.0.1:$BACKEND_PORT/health" >/dev/null 2>&1 &&
		curl -fsS -m 2 "http://127.0.0.1:$FRONTEND_PORT/" >/dev/null 2>&1; then
		log "site: http://127.0.0.1:$FRONTEND_PORT"
		break
	fi
	sleep 2
done

# ── Demo data ─────────────────────────────────────────────────
if [ "$SEED" = 1 ]; then
	log "seeding demo data"
	python3 -m pip install --quiet --disable-pip-version-check requests websockets >/dev/null 2>&1
	API="http://127.0.0.1:$BACKEND_PORT" python3 "$ROOT/scripts/seed.py" || log "seed skipped (see error above)"
fi
