# Deployment

English version of [../../operations/deployment.md](../../operations/deployment.md).

## Options

| Option | File | Note |
| --- | --- | --- |
| Docker Compose (recommended) | `docker-compose.yml` | one service, the `brainstorm-data` volume |
| Container by hand | `Dockerfile` | multi-stage, non-root `appuser` |
| Process manager | `Procfile`, `gunicorn.conf.py` | `gunicorn -c gunicorn.conf.py wsgi:app` |
| Manual run | `run.py` | development only |

## Procedure

```bash
cp .env.example .env
# fill in: SECRET_KEY (mandatory), ADMIN_SECRET_KEY, CHEAT_TESTER_CODE
make doctor
make compose-up
make smoke-test BASE_URL=http://localhost:5000
```

Compose fails when a mandatory secret is missing (`${VAR:?…}`) — protection against a
deployment with empty keys.

## Before production

1. **Rotate the secrets** if the repository was ever public: see
   [../security/secrets.md](../security/secrets.md).
2. `SESSION_COOKIE_SECURE=true` and HTTPS at the proxy.
3. `CORS_ORIGINS` — an explicit domain list instead of `*`.
4. `DEBUG=false` and `ENV=production` (otherwise tracebacks and verbose errors).
5. `SOCKETIO_ASYNC_MODE=eventlet` for load; `threading` is the fallback without extra
   dependencies.
6. A single process: rooms and the scheduler live in memory. Several replicas without
   shared state are not supported.
7. Back up the `brainstorm-data` volume, see
   [../database/storage.md](../database/storage.md).

## After deployment

```bash
make smoke-test BASE_URL=https://<domain>
curl -s https://<domain>/health | python -m json.tool
```
