# System context

English version of [../../architecture/system-context.md](../../architecture/system-context.md).

## What this is

BrainStorm is a web game/quiz with rooms, a leaderboard, a learning mode, campaigns,
user-generated questions and optional AI hints. One Flask + Socket.IO process, data in
SQLite inside `DATA_DIR`.

## Context diagram

```
   +------------+   +------------------+   +--------------------+
   | Player     |   | Host / admin     |   | Telegram / Min App |
   | (browser)  |   | (browser + code) |   | (optional)         |
   +-----+------+   +--------+---------+   +---------+----------+
         | HTTP/WebSocket    |                       |
         +---------+---------+-----------+-----------+
                   v                     v
        +--------------------------------------------+
        | Flask + Flask-SocketIO (gunicorn/eventlet) |
        |  app/domain      game rules and models     |
        |  app/http_api    HTTP routes               |
        |  app/realtime    rooms, chat, scheduler    |
        |  app/services    accounts, campaigns, UGC  |
        +-------+-----------------------+------------+
                |                       | optional
                v                       v
        +---------------+        +------------------+
        | SQLite        |        | GigaChat API     |
        | leaderboard.db|        | (questions, hints)|
        | user_data.db  |        +------------------+
        +---------------+
```

## Module boundaries

| Layer | Directory | Responsibility |
| --- | --- | --- |
| Domain | `app/domain/` | models, registry, game rules — no Flask, no network |
| HTTP | `app/http_api/` | routes, validation, access |
| Realtime | `app/realtime/` | rooms, chat, round scheduler |
| Services | `app/services/` | accounts, achievements, campaigns, leaderboard, UGC |
| Storage | `app/db.py`, `app/user_db.py` | SQLite, schema, WAL, idempotent migrations |
| External | `app/ai_client.py` | GigaChat; offline question bank without a key |

Rule: the domain imports no Flask, `http_api` holds no game rules.

## Failure modes of external systems

| Dependency | Mandatory | When missing |
| --- | --- | --- |
| Filesystem (`DATA_DIR`) | yes | the application cannot store data |
| GigaChat | no | the offline question bank is used |
| eventlet | no | `SOCKETIO_ASYNC_MODE=threading` |

## Supported scope

Declared: Python 3.12, Linux + Docker/Compose, a browser with WebSocket. Not declared:
several instances on a shared SQLite, clustered realtime (rooms live in process memory),
other DBMS.
