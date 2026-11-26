#!/usr/bin/env bash
# Keeps the dev stack alive while the container is running.
#   scripts/watchdog.sh            → loop forever, check every 20s
#   scripts/watchdog.sh --once     → single check (cron friendly)
#   scripts/watchdog.sh 5          → loop with a 5s interval
# Note: this cannot survive a container recycle — only a real host can (see DEPLOY.md).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_DIR="$ROOT/.run"
BACKEND_PORT="${PORT:-5000}"
FRONTEND_PORT="${FRONTEND_PORT:-12000}"
LOG="$RUN_DIR/watchdog.log"

mkdir -p "$RUN_DIR"

backend_up() { curl -fsS -m 3 "http://127.0.0.1:$BACKEND_PORT/health" >/dev/null 2>&1; }
frontend_up() { curl -fsS -m 3 "http://127.0.0.1:$FRONTEND_PORT/" >/dev/null 2>&1; }

check_once() {
	local restarted=0
	if ! backend_up; then
		echo "$(date -Is) backend down → bootstrap" >>"$LOG"
		PORT="$BACKEND_PORT" FRONTEND_PORT="$FRONTEND_PORT" "$ROOT/scripts/bootstrap.sh" >>"$LOG" 2>&1
		restarted=1
	fi
	if ! frontend_up; then
		echo "$(date -Is) frontend down → restart" >>"$LOG"
		(
			cd "$ROOT/frontend" || exit 1
			setsid nohup npx vite dev --port "$FRONTEND_PORT" --host 0.0.0.0 >>"$RUN_DIR/frontend.log" 2>&1 &
			echo $! >"$RUN_DIR/frontend.pid"
		)
		restarted=1
	fi
	return 0
}

if [ "${1:-}" = "--once" ]; then
	check_once
	exit 0
fi

INTERVAL="${1:-20}"
PIDFILE="$RUN_DIR/watchdog.pid"
if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE" 2>/dev/null)" 2>/dev/null; then
	echo "watchdog already running (pid $(cat "$PIDFILE"))"
	exit 0
fi
echo $$ >"$PIDFILE"
trap 'rm -f "$PIDFILE"' EXIT

echo "$(date -Is) watchdog started (interval ${INTERVAL}s)" >>"$LOG"
while true; do
	check_once
	sleep "$INTERVAL"
done
