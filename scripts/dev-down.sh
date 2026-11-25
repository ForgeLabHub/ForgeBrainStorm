#!/usr/bin/env bash
# Stop the backend and frontend started by scripts/dev-up.sh.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_DIR="$ROOT/.run"

stop() {
	local name="$1" pidfile="$RUN_DIR/$1.pid"
	if [ -f "$pidfile" ] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
		kill "$(cat "$pidfile")" && echo "$name: stopped"
	else
		echo "$name: no pidfile, nothing to stop"
	fi
	rm -f "$pidfile"
}

# The processes are detached with setsid, so fall back to a targeted match.
stop backend
pkill -f "target/debug/brainstorm" 2>/dev/null && echo "backend: matched leftover process"

stop frontend
pkill -f "vite dev --port" 2>/dev/null && echo "frontend: matched leftover process"

echo "done"
