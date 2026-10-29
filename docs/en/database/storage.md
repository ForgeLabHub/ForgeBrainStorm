# Data storage

English version of [../../database/storage.md](../../database/storage.md).

Two SQLite databases inside `DATA_DIR` (default `/app/data` in the container):

| File | Purpose | Writer |
| --- | --- | --- |
| `leaderboard.db` | anonymous game data: leaderboard, bans, room history | `app/db.py` |
| `user_data.db` | accounts: users, UGC questions, campaign progress, achievements | `app/user_db.py` |

## Rules

1. **WAL journaling.** Connections are short-lived and created per request: SQLite
   objects cannot be shared between eventlet green threads.
2. **The schema is created idempotently** through `PRAGMA table_info` introspection, not
   `ALTER TABLE ... except: pass`. A repeated start changes nothing.
3. **The two databases are separated deliberately:** an anonymous game must not require an
   account, and account data must not mix with game data.
4. **No external DBMS.** The declared boundary is one process plus SQLite; moving to
   PostgreSQL is a separate decision, not a "setting".

## Backup

```bash
# the files live in the brainstorm-data volume
docker run --rm -v brainstorm-data:/data -v "$PWD/backups:/backups" alpine \
  tar czf /backups/brainstorm-$(date -u +%Y%m%dT%H%M%SZ).tar.gz -C /data .
```

Restore: stop the container, unpack the archive into the volume, start it, check `/health`
(it reports the state of both databases).

## Verification

| Claim | How it is checked |
| --- | --- |
| The schema is created in an empty directory | `tests/conftest.py` uses a temporary `DATA_DIR` per run |
| Re-running the initialisation is safe | the application starts in CI, then the container restarts |
| Data survives a restart | the `brainstorm-data` volume plus `/health` after `docker compose restart` |
