# Архитектура ForgeBrainStorm

## Текущее состояние → Целевая структура

### Карта модулей

| Текущий путь | Целевой crate | Обоснование |
|--------------|---------------|-------------|
| `rust-server/src/domain/*` | `crates/domain` | Чистая логика без I/O — основа системы |
| `rust-server/src/services/ai/*` | `crates/ai` | AI-провайдеры, роутер, кэш, quality gate |
| `rust-server/src/services/*` (остальные) | `crates/server/src/services` | Бизнес-логика, БД, аудит |
| `rust-server/src/routes/*` | `crates/server/src/routes` | HTTP endpoints |
| `rust-server/src/ws/*` | `crates/server/src/ws` | WebSocket game loop |
| `rust-server/src/middleware/*` | `crates/server/src/middleware` | Auth, CORS, rate limit |
| `rust-server/src/config.rs` | `crates/server/src/config.rs` | Конфигурация сервера |
| `rust-server/src/error.rs` | `crates/server/src/error.rs` | AppError тип |
| `rust-server/src/main.rs` | `crates/server/src/main.rs` | Entry point |
| `rust-server/src/lib.rs` | `crates/server/src/lib.rs` | Library exports |
| `rust-server/migrations/` | `migrations/` | SQL миграции (общие) |
| НОВОЕ | `crates/protocol` | Общие типы: DTO, WS сообщения, ID newtypes, валидация |
| НОВОЕ | `crates/storage` | Репозитории: Postgres, Redis |
| НОВОЕ | `crates/cli` | CLI утилита `forge` |
| НОВОЕ | `crates/loadtest` | Load tester |
| НОВОЕ | `xtask/` | Cargo xtask для dev/bench/migrate |

### Правила зависимостей

```
protocol ← domain (только типы, никакого I/O)
storage  ← domain, protocol
ai       ← protocol (возвращает domain-типы)
server   → domain, storage, ai, protocol
cli      → server (dev), storage (migrate)
```

**Запрещено:**
- `domain` зависит от `tokio`/`axum`/`sqlx`/`redis`
- `protocol` зависит от чего-либо внутри workspace (только serde/validator)

## Фазы миграции

### Phase 1: Workspace skeleton (ПР-1.1)
- Создать `crates/`, `xtask/`
- Перенести `rust-server/` → `crates/server/`
- Workspace `Cargo.toml` с resolver = "2"
- `rust-toolchain.toml` (фиксированный toolchain)
- `.cargo/config.toml` (алиасы)
- CI обновлён для workspace
- **Никакой логики не двигаем**

### Phase 2: Extract protocol (ПР-1.2)
- Вынести DTO, WS-сообщения, ID newtypes
- `protocol` компилируется под `wasm32-unknown-unknown`
- Тесты сериализации (roundtrip)

### Phase 3: Extract domain (ПР-1.3)
- Room, Player, GameMode, PowerUp, scoring
- `#![deny(clippy::unwrap_used)]`
- Property-based тесты (proptest)
- Snapshot тесты (insta)

### Phase 4: Performance (ПР-1.4)
- Release profile оптимизации
- mimalloc global allocator
- Criterion бенчи
- CI bench job с порогом регрессии

### Phase 5: Persistence (ПР-2.x)
- Redis room store с rkyv snapshots
- Redis Streams для events + resume
- Graceful shutdown + reconnect
- WS binary protocol (postcard)

### Phase 6: Security & Observability (ПР-3.x)
- CSRF + strict cookies
- Answer rate-limit + server-side timing
- Prometheus metrics + /metrics
- Health endpoints (/livez, /readyz)
- Audit log expansion
- cargo-audit + cargo-deny в CI

### Phase 7: AI improvements (ПР-4.x)
- Dedup via fastembed
- Quality gate + token tracker
- SSE streaming generation
- L1/L2/L3 cache

### Phase 8: Quiz features (ПР-5.x)
- QuestionKind (multi/truefalse/numeric)
- Per-question stats + podium
- Share-PNG via resvg
- Spectator role
- Emote bar + reactions

## Принципы

1. **Один PR = одна тема** — не смешивать рефакторинг с фичами
2. **Тесты обязательны** — новая логика без тестов не принимается
3. **Никаких unwrap()/expect()** в prod-коде (кроме main и тестов)
4. **Документировать pub API** — `///` на pub элементах
5. **Conventional Commits** — feat:, fix:, refactor:, perf:, test:, docs:, chore:, build:, ci:
6. **Feature flags** — ломающие изменения за `#[cfg(feature = "...")]`
7. **Обоснование зависимостей** — каждый новый crate в описании PR

## Definition of Done (каждый PR)

- [ ] `cargo build --workspace` зелёный
- [ ] `cargo test --workspace` зелёный
- [ ] `cargo clippy --workspace -- -D warnings` зелёный
- [ ] `cargo fmt --check` зелёный
- [ ] Coverage не упал
- [ ] Публичный API задокументирован
- [ ] Обновлён CHANGELOG.md
- [ ] Обновлены docs/ при изменении поведения

## Метрики успеха

- **Domain coverage**: ≥85%
- **Server routes coverage**: ≥70%
- **WS echo p99**: <5ms
- **SubmitAnswer p99**: <500µs
- **Concurrent WS**: 20k на 8 vCPU
- **RAM/room**: <20KB
- **Cold start**: <150ms
