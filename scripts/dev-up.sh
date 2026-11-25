#!/usr/bin/env bash
# Start the Rust backend and the SvelteKit dev server in the background.
# Usage: scripts/dev-up.sh [port-for-frontend]
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_DIR="$ROOT/.run"
FRONTEND_PORT="${1:-12000}"
BACKEND_PORT="${PORT:-5000}"

mkdir -p "$RUN_DIR"

is_up() {
	curl -fsS -m 2 "http://127.0.0.1:$1" >/dev/null 2>&1
}

# 1. PostgreSQL
if ! pg_isready -q -h 127.0.0.1 -p 5432 2>/dev/null; then
	echo "postgres: not running, trying to start"
	(sudo service postgresql start || pg_ctlcluster 17 main start) >/dev/null 2>&1
	sleep 2
fi
pg_isready -q -h 127.0.0.1 -p 5432 && echo "postgres: ok" || echo "postgres: unavailable — games will fail to save results"

# 2. Backend
if is_up "$BACKEND_PORT/health"; then
	echo "backend: already running on $BACKEND_PORT"
else
	BIN="$ROOT/rust-server/target/debug/brainstorm"
	if command -v cargo >/dev/null 2>&1; then
		echo "backend: building"
		pushd "$ROOT/rust-server" >/dev/null || exit 1
		cargo build --quiet || { popd >/dev/null; exit 1; }
		popd >/dev/null || true
	elif [ -x "$BIN" ]; then
		echo "backend: cargo not installed, using existing binary"
	else
		echo "backend: cargo and prebuilt binary both missing"
		exit 1
	fi
	echo "backend: starting on $BACKEND_PORT"
	pushd "$ROOT/rust-server" >/dev/null || exit 1
	PORT="$BACKEND_PORT" setsid nohup "$BIN" >"$RUN_DIR/backend.log" 2>&1 </dev/null &
	echo $! >"$RUN_DIR/backend.pid"
	popd >/dev/null || true
	sleep 2
	is_up "$BACKEND_PORT/health" && echo "backend: ok" || echo "backend: check $RUN_DIR/backend.log"
fi

# 3. Frontend
if is_up "$FRONTEND_PORT"; then
	echo "frontend: already running on $FRONTEND_PORT"
else
	echo "frontend: starting on $FRONTEND_PORT"
	pushd "$ROOT/frontend" >/dev/null || exit 1
	setsid nohup npx vite dev --port "$FRONTEND_PORT" --host 0.0.0.0 >"$RUN_DIR/frontend.log" 2>&1 </dev/null &
	echo $! >"$RUN_DIR/frontend.pid"
	popd >/dev/null || true
	sleep 5
	is_up "$FRONTEND_PORT" && echo "frontend: ok" || echo "frontend: check $RUN_DIR/frontend.log"
fi

echo
echo "local:  http://127.0.0.1:$FRONTEND_PORT"
echo "logs:   $RUN_DIR/backend.log, $RUN_DIR/frontend.log"
