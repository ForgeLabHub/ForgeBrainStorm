# Monitoring

English version of [../../operations/monitoring.md](../../operations/monitoring.md).

| Signal | Where | Purpose |
| --- | --- | --- |
| `/health` | application | process state and both SQLite databases |
| Container HEALTHCHECK | `docker-compose.yml`, `Dockerfile` | restart on a hang |
| Logs | stdout, `app/logging_setup.py` | incident analysis, `LOG_LEVEL` |
| Socket.IO events | `app/realtime/*` | diagnosing rooms and rounds |
| Metrics | — | absent; this is a limitation, not an oversight |

## Minimal alerts

1. `/health` does not answer for 2 minutes or reports a database failure.
2. The container restarts more than once an hour.
3. The 5xx share is above 1% over 5 minutes.
4. GigaChat errors while a key is configured (AI degrades, the game keeps working).
5. A growing number of WebSocket disconnects — network problem or overload.

## What is missing

- No `/metrics` and no dashboards.
- No distributed tracing.
- No SLO expressed as a number.
- Rooms live in process memory: a restart loses active games (visible in logs, but not
  measured).
