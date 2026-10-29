#!/usr/bin/env bash
# Проверка живого инстанса BrainStorm.
#   ./scripts/smoke-test.sh [base-url]
set -uo pipefail
base="${1:-http://localhost:5000}"
failures=0
ok()  { printf '  \033[32mok\033[0m    %s\n' "$1"; }
bad() { printf '  \033[31mfail\033[0m  %s\n' "$1"; failures=$((failures+1)); }
code() { curl -s -o /dev/null -w '%{http_code}' -m 10 "$@"; }

printf '\nSmoke test: %s\n' "$base"
c="$(code "$base/health")"; [ "$c" = "200" ] && ok "GET /health -> 200" || bad "GET /health -> $c"
c="$(code "$base/")";       [ "$c" = "200" ] && ok "GET / -> 200" || bad "GET / -> $c"

body="$(curl -s -m 10 "$base/health")"
echo "  health: $body"
printf '\n'
[ "$failures" -gt 0 ] && { printf '\033[31mпровалено: %d\033[0m\n' "$failures"; exit 1; }
printf '\033[32mSmoke test пройден\033[0m\n'
