# BrainStorm — Rust Server

High-performance rewrite of the BrainStorm quiz platform backend in Rust.

## Status

**Phase 1**: Core infrastructure (this directory)
- ✅ Axum HTTP/WS server
- ✅ PostgreSQL with migrations
- ✅ Session-based auth (argon2id)
- ✅ Rate limiting
- ✅ CORS, tracing, error handling
- ✅ Room registry (in-memory)
- ⏳ WebSocket gameplay (partial)
- ⏳ GigaChat AI integration (scaffolded)
- ⏳ Frontend (separate SvelteKit project)

## Quick Start

### Prerequisites

- Rust 1.80+
- Docker & Docker Compose (for Postgres/Redis)
- GigaChat API credentials (optional, for AI questions)

### Local Development

1. **Start Postgres and Redis**:
   ```bash
   docker compose up -d postgres redis
   ```

2. **Run migrations**:
   ```bash
   cargo install sqlx-cli
   sqlx migrate run --source migrations
   ```
   Or let the app run them automatically on startup.

3. **Configure environment**:
   ```bash
   cp config/default.toml config/local.toml
   # Edit local.toml with your settings
   export BRAINSTORM_SECRET_KEY="your-secret-key-at-least-32-bytes"
   export BRAINSTORM_ADMIN_SECRET_KEY="your-admin-key"
   export GIGACHAT_CREDENTIALS="your-gigachat-credentials"
   ```

4. **Run the server**:
   ```bash
   cargo run
   ```
   Server starts at `http://localhost:5000`.

5. **Health check**:
   ```bash
   curl http://localhost:5000/health
   curl http://localhost:5000/readyz
   ```

### Docker Deployment

```bash
docker compose up --build
```

Access the API at `http://localhost:5000`.

## API Endpoints

### Health
- `GET /health` — liveness probe
- `GET /readyz` — readiness probe (checks DB)

### Auth
- `POST /api/auth/register` — create account
- `POST /api/auth/login` — authenticate
- `POST /api/auth/logout` — end session
- `GET /api/auth/me` — current user

### Admin
- `POST /api/admin/activate` — activate admin mode
- `GET /api/admin/rooms` — list rooms
- `GET /api/admin/users` — list users
- `POST /api/admin/ban` — ban user/IP
- `POST /api/admin/unban` — remove ban
- `POST /api/admin/reset_user` — reset stats
- `GET /api/admin/audit_log` — view audit log

### Leaderboard
- `GET /api/leaderboard/top?limit=50` — top players
- `GET /api/leaderboard/rank/:username` — user rank

### Rooms
- `GET /api/rooms` — list public rooms
- `POST /api/rooms` — create room
- `GET /api/rooms/:code` — room details

### WebSocket
- `WS /ws` — real-time gameplay (auth required)

## Configuration

All settings can be configured via environment variables (prefixed `BRAINSTORM_`) or `config/default.toml`.

**Critical production settings**:
- `BRAINSTORM_ENV=production` — enables strict validation
- `BRAINSTORM_SECRET_KEY` — ≥32 bytes, not the dev default
- `BRAINSTORM_ADMIN_SECRET_KEY` — required in production
- `BRAINSTORM_SESSION_COOKIE_SECURE=true` — required in production
- `BRAINSTORM_CORS_ORIGINS` — explicit list, no `*` in production

See `config/default.toml` for all options.

## Security

### Implemented
- ✅ Argon2id password hashing
- ✅ Session tokens in secure, httpOnly cookies
- ✅ Constant-time string comparison
- ✅ HMAC for rejoin tokens (scaffolded)
- ✅ Rate limiting (per-IP token bucket)
- ✅ CORS allowlist
- ✅ Audit log for admin actions
- ✅ Input validation (validator crate)
- ✅ SQL injection prevention (sqlx bind parameters)

### TODO
- ⏳ CSRF tokens for state-changing endpoints
- ⏳ Rejoin token verification
- ⏳ IP-based bans in WebSocket layer
- ⏳ Tester mode with explicit grant (not global code)

## Performance Targets

- p50 HTTP latency: <30ms
- p99 HTTP latency: <200ms
- WebSocket broadcast (100 players): <50ms
- Concurrent connections: 1000+
- Memory footprint: <100MB (idle)

## Architecture

```
src/
├── main.rs              # Entry point, server bootstrap
├── config.rs            # Configuration loading & validation
├── error.rs             # Error types (thiserror)
├── db.rs                # Postgres pool, migrations
├── security.rs          # HMAC, constant-time utils
├── domain/              # Core models (Room, Player)
│   ├── room.rs          # Room state machine, registry
│   └── player.rs        # Player model
├── services/            # Business logic
│   ├── accounts.rs      # User management
│   ├── ai_client.rs     # GigaChat integration
│   ├── audit.rs         # Audit log
│   └── leaderboard.rs   # Ranking aggregates
├── middleware/          # HTTP middleware
│   ├── auth.rs          # Session validation
│   ├── cors.rs          # CORS layer
│   └── rate_limit.rs    # Token bucket
├── routes/              # HTTP handlers
│   ├── admin.rs         # Admin endpoints
│   ├── auth.rs          # Auth endpoints
│   ├── health.rs        # Health probes
│   ├── leaderboard.rs   # Ranking endpoints
│   └── rooms.rs         # Room CRUD
└── ws/                  # WebSocket handler
    └── mod.rs           # WS protocol (partial)

migrations/
└── 20240101000001_initial.sql  # Database schema
```

## Migration from Python

This Rust server is designed to run **in parallel** with the existing Python backend during migration. The strategy:

1. **Weeks 1-2**: Core infrastructure (auth, health, rooms) — **done**
2. **Weeks 3-4**: Complete WebSocket gameplay + AI integration
3. **Week 5**: Feature parity with Python (all endpoints)
4. **Week 6**: Load testing, security review
5. **Week 7**: Cutover to Rust, Python → legacy/
6. **Week 8**: Cleanup, documentation, monitoring

See the parent repository's audit report for the full migration plan.

## Testing

```bash
# Unit tests
cargo test

# Integration tests (requires Postgres)
docker compose up -d postgres
cargo test --test health -- --ignored

# Lint
cargo clippy -- -D warnings
cargo fmt --check
```

## License

MIT
