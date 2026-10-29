# Module map: what to borrow for another project

The application is already layered. The table answers "what to take if I need X".

| Need | Files to take | External dependencies | Replace |
| --- | --- | --- | --- |
| Pure game rules and scoring | `app/domain/` | — | the rule set itself; keep it free of I/O |
| Rooms, chat, round scheduler | `app/realtime/` | `flask-socketio` | transport and event names |
| Accounts, achievements, campaigns | `app/services/` | — | tables in `app/user_db.py` |
| SQLite with WAL and idempotent migrations | `app/db.py`, `app/user_db.py` | — | schema functions |
| Optional AI provider with offline fallback | `app/ai_client.py`, `app/services/ai_client.py` | `gigachat` | provider and prompts |
| Input validation and error format | `app/validation.py`, `app/errors.py` | — | message catalogue |
| Rate limiting in-process | `app/ratelimit.py` | — | limits per endpoint |
| Health endpoint covering both databases | `app/http_api/health.py` | — | database list |

## Transfer rules

1. The dependency direction is fixed: `domain` → nothing, `services` → `domain` + `db`,
   `realtime`/`http_api` → `services`. A module that breaks the arrow cannot be borrowed
   alone.
2. No global mutable state except the process-local room registry: it is the reason a
   single process is the supported deployment.
3. Tests come with the module: `domain` → `tests/test_domain.py`, `realtime` →
   `tests/test_realtime.py`, `tests/test_socket_flow.py`.
4. Configuration is read through one helper, never at import time.

## What is not here

- No `Repository`/ORM layer: two SQLite files with explicit SQL.
- No plugin system: services are imported directly, the graph is small enough to read.
