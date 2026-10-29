# BrainStorm — English overview

English overview. The full Russian README is [../../README.md](../../README.md).

## What it is

A web quiz game: rooms, a leaderboard, a learning mode, campaigns, user-generated
questions and optional AI hints (GigaChat). One Flask + Socket.IO process, data in SQLite
inside `DATA_DIR`.

## Run it

```bash
cp .env.example .env         # SECRET_KEY is mandatory; AI is optional
make doctor
make install-dev
make test                    # 190 tests, no external services
python run.py                # http://localhost:5000
# or: ./forge.sh start   (linux/macos)   forge.bat start   (windows)
```

## What is inside

| Layer | Directory | Responsibility |
| --- | --- | --- |
| Domain | `app/domain/` | models, registry, game rules — no Flask, no network |
| HTTP | `app/http_api/` | routes, validation, access |
| Realtime | `app/realtime/` | rooms, chat, round scheduler |
| Services | `app/services/` | accounts, achievements, campaigns, leaderboard, UGC |
| Storage | `app/db.py`, `app/user_db.py` | SQLite, schema, WAL, idempotent migrations |
| External | `app/ai_client.py` | GigaChat; offline question bank without a key |

Borrowing map: [modules/module-map.md](modules/module-map.md).

## Documentation

[index.md](index.md) · [BUILD.md](BUILD.md) · [verification/matrix.md](verification/matrix.md)
· PDF of every document in [../pdf/](../pdf/).

## Licence

MIT — see [../../LICENSE](../../LICENSE).
