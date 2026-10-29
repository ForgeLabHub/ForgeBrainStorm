#!/usr/bin/env bash
# Проверка окружения. Ничего не устанавливает и не меняет.
set -uo pipefail
expected_python="3.12"
failures=0; warnings=0
ok()   { printf '  \033[32mok\033[0m    %s\n' "$1"; }
bad()  { printf '  \033[31mfail\033[0m  %s\n' "$1"; failures=$((failures+1)); }
warn() { printf '  \033[33mwarn\033[0m  %s\n' "$1"; warnings=$((warnings+1)); }

printf '\nИнструменты\n'
if command -v python3 >/dev/null 2>&1; then
  pv="$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
  [ "$pv" = "$expected_python" ] && ok "python $pv" || warn "python $pv, образ использует $expected_python"
else
  bad "python3 не найден"
fi
command -v git >/dev/null 2>&1 && ok "git" || bad "git не найден"
command -v docker >/dev/null 2>&1 && ok "docker" || warn "docker не найден — доступны только уровни A"

printf '\nСекреты\n'
if git ls-files --error-unmatch .env >/dev/null 2>&1; then
  bad ".env под контролем версий — удалите из индекса (git rm --cached .env)"
else
  ok ".env не отслеживается git"
fi
if [ -f .env ]; then
  grep -qE '^SECRET_KEY=.+' .env && ok "SECRET_KEY задан локально" || bad "SECRET_KEY пуст"
  if grep -qE '^GIGACHAT_CREDENTIALS=(YOUR_BASE64.*|)$' .env; then
    warn "GIGACHAT_CREDENTIALS не задан — AI-функции выключены, остальное работает"
  else
    ok "GIGACHAT_CREDENTIALS задан"
  fi
else
  warn ".env отсутствует: cp .env.example .env (AI при этом выключен)"
fi

printf '\nЗависимости\n'
python3 - <<'PY' 2>/dev/null || warn "зависимости не установлены: make install"
import importlib.util
missing = [m for m in ("flask", "flask_socketio", "dotenv") if not importlib.util.find_spec(m)]
if missing:
    raise SystemExit(1)
print("  \033[32mok\033[0m    flask, flask-socketio, python-dotenv установлены")
PY

printf '\nПорт\n'
if command -v ss >/dev/null 2>&1 && ss -ltn 2>/dev/null | grep -q ":5000 "; then warn "порт 5000 занят"; else ok "порт 5000 свободен"; fi

printf '\n'
if [ "$failures" -gt 0 ]; then printf '\033[31m%d проверок провалено\033[0m, предупреждений: %d\n' "$failures" "$warnings"; exit 1; fi
printf '\033[32mГотово к работе\033[0m, предупреждений: %d\n' "$warnings"
