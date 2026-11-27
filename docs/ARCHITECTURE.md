# Карта архитектуры: текущее состояние → целевая структура

## Текущая структура (`rust-server/`)

```
rust-server/
├── src/
│   ├── main.rs              # Entry point → server
│   ├── config.rs            # Figment config → server
│   ├── db.rs                # PgPool + migrations → server
│   ├── error.rs             # AppError → server
│   ├── middleware/          # auth, cors, rate_limit → server
│   ├── routes/              # HTTP handlers → server
│   │   ├── auth.rs
│   │   ├── rooms.rs
│   │   ├── ai.rs
│   │   ├── leaderboard.rs
│   │   ├── profile.rs
│   │   ├── admin.rs
│   │   └── health.rs
│   ├── ws/                  # WebSocket game loop → server
│   ├── domain/              # Pure logic (НО с I/O зависимостями!) → domain
│   │   ├── room.rs
│   │   ├── player.rs
│   │   ├── mode.rs
│   │   ├── power_up.rs
│   │   ├── role.rs
│   │   └── cheat.rs
│   └── services/            # I/O and external systems
│       ├── accounts.rs      → storage (postgres)
│       ├── history.rs       → storage (postgres)
│       ├── leaderboard.rs   → storage (postgres)
│       ├── audit.rs         → storage (postgres)
│       ├── questions.rs     → ai
│       └── ai/              → ai
│           ├── router.rs
│           ├── cache.rs
│           └── providers/
│               ├── openai.rs
│               ├── gigachat.rs
│               └── ollama.rs
├── migrations/              → migrations/ (workspace root)
└── tests/                   → crates/server/tests/
```

## Целевая структура (workspace)

```
forgebrain/
├── Cargo.toml               # [workspace], resolver 2
├── rust-toolchain.toml      # pinned toolchain
├── .cargo/config.toml       # aliases
├── crates/
│   ├── protocol/            # Общие типы (backend + wasm)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── messages.rs  # WS-протокол (postcard-ready)
│   │   │   ├── dto.rs       # request/response DTO
│   │   │   ├── ids.rs       # newtype'ы: RoomCode, UserId, Qid
│   │   │   └── validation.rs
│   │   └── Cargo.toml
│   │
│   ├── domain/              # Чистая логика, БЕЗ tokio/axum/sqlx
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── room.rs
│   │   │   ├── player.rs
│   │   │   ├── mode.rs
│   │   │   ├── powerup.rs
│   │   │   └── scoring/
│   │   └── Cargo.toml
│   │
│   ├── storage/             # Репозитории: Postgres, Redis
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── postgres/
│   │   │   └── redis/
│   │   └── Cargo.toml
│   │
│   ├── ai/                  # AI провайдеры + router + cache
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── router.rs
│   │   │   ├── cache.rs
│   │   │   ├── quality.rs
│   │   │   └── providers/
│   │   │       ├── openai.rs
│   │   │       ├── gigachat.rs
│   │   │       └── ollama.rs
│   │   └── Cargo.toml
│   │
│   └── server/              # axum + ws + services (собирает всё)
│       ├── src/
│       │   ├── main.rs
│       │   ├── config.rs
│       │   ├── state.rs
│       │   ├── error.rs
│       │   ├── middleware/
│       │   ├── routes/
│       │   ├── ws/
│       │   └── services/
│       └── Cargo.toml
│
├── migrations/              # sqlx migrations (из rust-server/migrations)
└── docs/
    ├── ARCHITECTURE.md
    ├── PROTOCOL.md
    └── adr/
```

## Правила зависимостей

```
protocol ← domain (только типы, никакого I/O)
storage  ← domain, protocol
ai       ← protocol (возвращает domain-типы)
server   → domain, storage, ai, protocol
```

**Запрещено:**
- `domain` зависит от `tokio`/`axum`/`sqlx`/`redis`
- `protocol` зависит от чего-либо внутри workspace (только serde/validator)

## Карта миграции файлов

| Текущий путь | Целевой crate | Обоснование |
|--------------|---------------|-------------|
| `rust-server/src/domain/*` | `crates/domain` | Чистая логика, но нужно убрать I/O зависимости |
| `rust-server/src/services/ai/*` | `crates/ai` | AI провайдеры, router, cache |
| `rust-server/src/services/{accounts,history,leaderboard,audit}.rs` | `crates/storage` | Postgres репозитории |
| `rust-server/src/routes/*` | `crates/server` | HTTP handlers |
| `rust-server/src/ws/*` | `crates/server` | WebSocket game loop |
| `rust-server/src/middleware/*` | `crates/server` | Auth, CORS, rate limit |
| `rust-server/src/config.rs` | `crates/server` | Конфигурация |
| `rust-server/src/error.rs` | `crates/server` | AppError |
| `rust-server/src/main.rs` | `crates/server` | Entry point |
| `rust-server/migrations/*` | `migrations/` | SQL миграции (workspace root) |

## Фазы миграции

### Phase 1: Workspace skeleton (ПР-1.1)
- ✅ Создать `crates/`, `xtask/`
- ✅ Перенести `rust-server/` → `crates/server/`
- ✅ Workspace `Cargo.toml`, `rust-toolchain.toml`, `.cargo/config.toml`
- ✅ CI обновлён
- **Никакой логики не двигаем**

### Phase 2: Extract protocol (ПР-1.2)
- Вынести DTO, WS-сообщения, ID newtypes
- `protocol` компилируется под `wasm32-unknown-unknown`
- Тесты сериализации (roundtrip)

### Phase 3: Extract domain (ПР-1.3)
- Room, Player, GameMode, PowerUp
- **Убрать I/O зависимости из domain**
- `#![deny(clippy::unwrap_used)]`
- Property-based тесты (proptest)

### Phase 4: Extract storage (ПР-2.1)
- accounts, history, leaderboard, audit → `crates/storage`
- Postgres + Redis repositories

### Phase 5: Extract ai (ПР-4.x)
- AI providers, router, cache → `crates/ai`

### Phase 6: Performance (ПР-1.4)
- Release profile optimizations
- mimalloc global allocator
- Criterion benchmarks

## Проблемы текущего `domain/`

Текущий `domain/` **не является чисто domain слоем**:
- `room.rs` имеет зависимости на `tokio::time::Instant`
- `player.rs` имеет `outbox: Option<tokio::sync::mpsc::Sender<String>>`
- `mode.rs` имеет зависимости на `serde_json::Value`

**Решение в PR-1.3:**
1. Заменить `Instant` на `Duration` или собственный тип
2. Заменить `mpsc::Sender` на trait или event queue
3. Вынести JSON serialization в `protocol` crate

## Метрики успеха

- **Domain coverage**: ≥85%
- **Server routes coverage**: ≥70%
- **Workspace builds**: все crates компилируются независимо
- **Test time**: <30s для всего workspace
- **Cold start**: <150ms
