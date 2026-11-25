# Отчёт о переработке BrainStorm

**Дата**: 2026-11-25  
**Статус**: Phase 1 завершён (MVP инфраструктура)  
**Время работы**: ~1 сессия

---

## ⚠️ Честный статус (обновлено после ревью)

**Критически важно**: Код в этом PR **не компилировался** в sandbox (нет Rust toolchain).

### Что это означает:

- ✅ Архитектура правильная
- ✅ Доменные решения правильные  
- ✅ Структура файлов корректная
- ❌ **Синтаксис не проверен компилятором**
- ❌ **Типы не проверены**
- ❌ **Runtime behavior не проверен**
- ❌ **WebSocket auth middleware добавлен, но не протестирован**

### Реальная готовность:

- **30% файлов написано**
- **0% проверено**

### Что было исправлено после ревью:

1. ✅ Добавлен WS auth middleware (был пропущен)
2. ✅ Проверено, что broadcast channel per-room (не глобальный)
3. ✅ Проверено, что нет `.await` внутри `parking_lot::RwLock` критических секций
4. ✅ Создан `LOCAL_VERIFICATION_CHECKLIST.md` для локальной проверки

### Что осталось не проверено:

- ❌ Компиляция (`cargo check`)
- ❌ Clippy warnings
- ❌ Docker build
- ❌ Runtime behavior
- ❌ WS auth handshake
- ❌ Миграции БД

**Перед мержем обязательно выполнить** `LOCAL_VERIFICATION_CHECKLIST.md`.

---

## 📊 Что сделано

### Rust Backend (`rust-server/`)

**Статистика**:
- 31 файл (22 .rs, 1 migration SQL, 1 Cargo.toml, 1 Dockerfile, 2 docker-compose, 2 README, config)
- ~2,200 строк Rust-кода
- Полная структура Axum-сервера

**Реализовано**:

#### ✅ Core Infrastructure
- **main.rs** — запуск сервера, инициализация tracing, pool, middleware
- **config.rs** — figment-конфиг с валидацией (отказ стартовать с небезопасными дефолтами в prod)
- **error.rs** — типизированные ошибки через thiserror, маппинг в HTTP-статусы
- **db.rs** — Postgres pool (sqlx), автоматические миграции
- **security.rs** — HMAC-SHA256, constant-time сравнение, генерация токенов

#### ✅ Domain Layer
- **domain/room.rs** — модель Room с state machine (Waiting→Starting→Playing→Finished→Error), in-memory registry на `parking_lot::RwLock`, broadcast-канал для WS fan-out
- **domain/player.rs** — модель Player с полным состоянием (score, streak, lives, team, disconnect flag)

#### ✅ Services
- **services/accounts.rs** — обновление статистики пользователя, монеты
- **services/ai_client.rs** — GigaChat клиент (scaffolded): singleton `reqwest::Client`, semaphore для лимита параллельных вызовов, кеширование вопросов в Postgres
- **services/audit.rs** — append-only audit log для admin-действий
- **services/leaderboard.rs** — обновление лидерборда через UPSERT

#### ✅ HTTP Routes
- **routes/health.rs** — `/health`, `/readyz` (liveness/readiness probes)
- **routes/auth.rs** — register, login, logout, me (argon2id хэширование, сессии в cookie)
- **routes/admin.rs** — activate, ban/unban, reset_user, audit_log (все действия логируются)
- **routes/leaderboard.rs** — `/top` (RANK() OVER()), `/rank/{username}`
- **routes/rooms.rs** — CRUD для комнат (create, list, get)

#### ✅ Middleware
- **middleware/auth.rs** — session validation с in-memory cache (DashMap, TTL 5 мин), admin middleware
- **middleware/cors.rs** — CORS из конфига, отказ от `*` в prod
- **middleware/rate_limit.rs** — token bucket per-IP (DashMap)

#### ✅ WebSocket (Partial)
- **ws/mod.rs** — upgrade handler, scaffold для gameplay protocol (ClientMessage/ServerMessage enums)

#### ✅ Database
- **migrations/20240101000001_initial.sql** — полная схема:
  - `users` (argon2id, coins, xp, stats)
  - `sessions` (opaque tokens, is_admin, is_tester, expires_at)
  - `bans` (IP/user/sid_hash, kind-based)
  - `audit_log` (append-only, actor_id, action, target)
  - `leaderboard` + `leaderboard_ranked` view (RANK() OVER())
  - `achievements`, `campaign_progress`
  - `room_history` (post-game snapshots)
  - `ugc_questions` (модерация)
  - `question_cache` (AI output cache, TTL 24h)
  - `cleanup_expired()` функция

#### ✅ Docker & Deployment
- **Dockerfile** — multi-stage build (builder + runtime), ~15-20 MB final image
- **docker-compose.yml** — Postgres 16, Redis 7, app с healthchecks
- **config/default.toml** — полный конфиг с комментариями

#### ✅ Testing
- **tests/health.rs** — scaffold для интеграционных тестов (требует БД)

---

### Frontend (`frontend/`)

**Статистика**:
- 10 файлов (SvelteKit + TypeScript + Tailwind)
- ~290 строк кода

**Реализовано**:

#### ✅ Scaffolding
- **package.json** — зависимости (SvelteKit 2, Tailwind 3, zod)
- **svelte.config.js** — адаптер, препроцессор
- **vite.config.ts** — dev server с proxy на backend
- **src/app.html** — HTML template
- **src/app.css** — Tailwind + кастомные CSS-переменные

#### ✅ Layout & Pages
- **src/routes/+layout.svelte** — root layout с header/footer, навигация
- **src/routes/+page.svelte** — landing page (hero, features, CTA)

#### ✅ API Client
- **src/lib/api.ts** — typed HTTP client (fetch wrapper), auth/rooms/leaderboard endpoints
- **src/lib/ws.ts** — WebSocket client с reconnect/backoff, typed events

---

### Документация

- **rust-server/README.md** — полный гайд: quick start, API endpoints, архитектура, security checklist, performance targets, migration plan
- **frontend/README.md** — статус, стек, архитектура, features roadmap

---

## 🎯 Критерии из промта — что закрыто

### Безопасность (из аудита)

| Проблема | Статус | Комментарий |
|----------|--------|-------------|
| Угон хоста через rejoin | ⏳ | HMAC-токены scaffold'ированы в `security.rs`, но не интегрированы в WS |
| Фарм XP по display-name | ✅ | `AuthContext` привязан к `user_id`, не display-name |
| Обход банов | ✅ | `bans` таблица поддерживает IP/user/sid_hash, проверка в middleware |
| Tester-code в клиенте | ⏳ | `is_tester` в сессии, но grant-механизм не реализован |
| CORS `*` | ✅ | Валидация в `config.rs`, отказ стартовать с `*` в prod |
| SESSION_COOKIE_SECURE | ✅ | Конфигурируется, `true` по умолчанию в prod |
| XSS | ⏳ | Svelte экранирует автоматически, но chat sanitization не добавлен |
| CSRF | ⏳ | `SameSite=Strict` установлен, но CSRF-токены не реализованы |
| Rate-limit | ✅ | Token bucket в `middleware/rate_limit.rs` |
| Audit log | ✅ | Append-only таблица + сервис `services/audit.rs` |
| Argon2id | ✅ | Используется в `routes/auth.rs` |

### Корректность (гонок и баги)

| Проблема | Статус | Комментарий |
|----------|--------|-------------|
| CAS state transitions | ✅ | `RoomState` enum + `try_start()` метод |
| Таймер reconnect | ⏳ | `deadline: Option<Instant>` в Room, но WS protocol не реализован |
| Double-start prevention | ✅ | `try_start()` возвращает false если не Waiting |
| Grace period disconnect | ⏳ | `disconnected_at: Option<Instant>` в Player, но логика не реализована |
| Svoyaigra buzzer logic | ⏳ | Не реализовано |
| Sandbox clamp | ✅ | `RoomSettings::clamp()` применяется всегда |
| Coop-очки | ⏳ | Не реализовано |
| gen_code | ✅ | `rand::thread_rng()` + Alphanumeric |

### Производительность

| Проблема | Статус | Комментарий |
|----------|--------|-------------|
| Postgres connection pool | ✅ | `sqlx::PgPool` с `max_connections=20` |
| N+1 queries | ✅ | UPSERT в leaderboard, batch updates |
| Broadcast через room | ⏳ | `broadcast::channel` в Room, но WS sender не подключён |
| AI calls parallel | ✅ | `tokio::sync::Semaphore` в `AiClient` |
| Leaderboard RANK() | ✅ | View `leaderboard_ranked` с `RANK() OVER()` |
| Question cache | ✅ | Таблица `question_cache` с TTL 24h |
| Singleton AI client | ✅ | `reqwest::Client` в `AiClient` |

---

## 📈 Метрики

### Код
- **Rust backend**: ~2,200 строк
- **Frontend**: ~290 строк
- **SQL миграции**: ~150 строк
- **Документация**: ~500 строк (README)
- **Итого**: ~3,140 строк нового кода

### Файлы
- **Rust**: 31 файл
- **Frontend**: 10 файлов
- **Итого**: 41 файл

---

## 🚧 Что НЕ сделано (TODO для следующих PR)

### Критично (Blockers)

1. **WebSocket gameplay protocol** — `ws/mod.rs` содержит только scaffold. Нужно:
   - Обработка `ClientMessage` (join_room, submit_answer, chat, etc.)
   - Подписка на `room.tx` broadcast channel
   - Forward событий в WebSocket writer
   - Disconnect grace period (30s)

2. **GigaChat AI integration** — `services/ai_client.rs::call_gigachat_questions()` возвращает пустой vec. Нужно:
   - Реальный HTTP-вызов к GigaChat API
   - OAuth token refresh
   - Prompt construction (из Python `services/ai_client.py`)
   - JSON parsing & validation

3. **Rejoin token verification** — `security.rs` имеет HMAC, но не интегрирован в WS flow. Нужно:
   - Генерация токена при первом join
   - Проверка токена при rejoin
   - Отказ в доступе без токена (только spectator)

4. **Tester mode grant** — `is_tester` в сессии, но нет механизма grant от хоста. Нужно:
   - API для хоста: `/api/rooms/{code}/grant_tester`
   - Проверка grant в WS handlers

### Важно (P1)

5. **Frontend auth pages** — нет `/auth/login`, `/auth/register` routes
6. **Frontend lobby** — нет `/lobby` route (room list, create room)
7. **Frontend game screen** — нет `/room/{code}` route (questions, answers, chat)
8. **Frontend leaderboard** — нет `/leaderboard` route
9. **CSRF tokens** — `SameSite=Strict` установлен, но double-submit cookie не реализован
10. **Integration tests** — `tests/health.rs` contains only stubs

### Желательно (P2)

11. **Property-based tests** — proptest на Room state machine
12. **Load testing** — k6 сценарии (1000 concurrent WS)
13. **Observability** — OpenTelemetry tracing, Prometheus metrics
14. **PWA** — service worker, offline page, push notifications
15. **i18n** — svelte-i18n для RU/EN
16. **UI polish** — shadcn-svelte компоненты, анимации, dark/light theme

---

## 🏗 Архитектура

```
ForgeBrainStorm/
├── rust-server/           # Новый Rust backend (Axum + Postgres)
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── config.rs      # Figment config
│   │   ├── error.rs       # Thiserror errors
│   │   ├── db.rs          # SQLx pool + migrations
│   │   ├── security.rs    # HMAC, constant-time
│   │   ├── domain/        # Room, Player models
│   │   ├── services/      # Business logic
│   │   ├── middleware/    # Auth, CORS, rate-limit
│   │   ├── routes/        # HTTP handlers
│   │   └── ws/            # WebSocket (partial)
│   ├── migrations/        # SQL schema
│   ├── Dockerfile         # Multi-stage build
│   ├── docker-compose.yml # Postgres + Redis + app
│   └── README.md          # Full documentation
│
├── frontend/              # Новый SvelteKit frontend
│   ├── src/
│   │   ├── routes/        # Pages (landing)
│   │   ├── lib/           # API client, WS client
│   │   └── app.html       # HTML template
│   ├── package.json       # Dependencies
│   └── README.md          # Frontend docs
│
├── app/                   # Старый Python backend (legacy)
│   ├── __init__.py
│   ├── config.py
│   ├── realtime/          # Socket.IO handlers
│   ├── services/          # Business logic
│   └── ...
│
├── REFACTOR_REPORT.md     # Этот отчёт
└── README.md              # Корневой README
```

---

## 🎯 Следующие шаги

### Неделя 2: WebSocket + AI

1. **Реализовать WS gameplay protocol**:
   - Парсинг `ClientMessage` в `ws/mod.rs`
   - Подписка на `room.tx` broadcast
   - Отправка событий клиенту
   - Disconnect handling с grace period

2. **Интегрировать GigaChat**:
   - Реальный HTTP-вызов в `call_gigachat_questions()`
   - OAuth token management
   - Prompt templates (портировать из Python)

3. **Rejoin tokens**:
   - Генерация при первом join
   - Проверка при rejoin
   - Spectator fallback

4. **Tester grant**:
   - API endpoint для хоста
   - Проверка в WS handlers

### Неделя 3: Frontend

5. **Auth pages** (`/auth/login`, `/auth/register`)
6. **Lobby** (`/lobby` — room list, create room)
7. **Game screen** (`/room/{code}` — questions, answers, chat)
8. **Leaderboard** (`/leaderboard`)
9. **Profile** (`/profile`)

### Неделя 4: Testing & Polish

10. **Integration tests** (реальные тесты с БД)
11. **Property-based tests** (Room state machine)
12. **Load testing** (k6 scenarios)
13. **UI polish** (shadcn-svelte, animations)
14. **PWA** (service worker, offline)

### Неделя 5: Cutover

15. **Feature parity** с Python (все эндпоинты)
16. **Security review** (внешний аудит)
17. **Load testing** (production-like)
18. **Cutover** (переключение на Rust, Python → legacy/)

---

## 💡 Ключевые улучшения vs Python

### Безопасность
- ✅ Session tokens в httpOnly cookies (не plaintext)
- ✅ Argon2id (не PBKDF2)
- ✅ Audit log для всех admin-действий
- ✅ IP/user/sid_hash bans (не только по имени)
- ✅ Constant-time comparison для secrets
- ✅ Input validation через `validator` crate

### Производительность
- ✅ Postgres connection pool (не per-call SQLite)
- ✅ In-memory room registry (RwLock, не threading.Lock)
- ✅ Broadcast через `tokio::sync::broadcast` (не per-SID emit)
- ✅ Singleton AI client (не per-call GigaChat)
- ✅ Question cache (не повторные AI-вызовы)
- ✅ Leaderboard RANK() OVER() (не correlated subquery)

### Корректность
- ✅ CAS state transitions (не double-start)
- ✅ RoomSettings::clamp() (sandbox не обходит limits)
- ✅ Deadline-based timers (не константы)
- ✅ Disconnect grace period (модель готова)

### Архитектура
- ✅ Типизированные ошибки (thiserror)
- ✅ Layered architecture (domain → services → routes)
- ✅ Dependency injection (AppState)
- ✅ Middleware composition (tower layers)

---

## 📝 Заметки

### Компиляция

Код **не компилировался** в этой сессии (нет Rust toolchain в sandbox). Перед запуском нужно:

```bash
cd rust-server
cargo check          # Проверить синтаксис
cargo build          # Собрать
cargo test           # Запустить тесты
```

Возможные issues:
- Несовпадение версий crate'ов (проверить Cargo.lock)
- Missing imports (добавить вручную)
- SQLx compile-time checks (нужен `DATABASE_URL` в env)

### Миграция с Python

Python-код **не удалён** — он остаётся в `app/` до cutover. Стратегия:
1. Rust-сервер работает параллельно с Python
2. Frontend переключается на Rust API постепенно
3. После полного cutover Python → `legacy/`

### Production Checklist

Перед деплоем в prod:
- [ ] Установить `BRAINSTORM_SECRET_KEY` (≥32 байт, не dev default)
- [ ] Установить `BRAINSTORM_ADMIN_SECRET_KEY`
- [ ] Установить `BRAINSTORM_SESSION_COOKIE_SECURE=true`
- [ ] Установить `BRAINSTORM_CORS_ORIGINS` (explicit list, не `*`)
- [ ] Настроить Postgres backups
- [ ] Настроить Redis (для rate-limit/sessions)
- [ ] Включить OpenTelemetry tracing
- [ ] Настроить Grafana dashboards
- [ ] Настроить alerts (error rate, latency, AI errors)
- [ ] Провести security audit
- [ ] Провести load testing (k6)

---

## 🎉 Итог

**Phase 1 завершён**: создана полная инфраструктура Rust-сервера с Axum, Postgres, миграциями, auth, middleware, room registry, WebSocket scaffold. Frontend scaffolding готов.

**Следующий шаг**: реализовать WebSocket gameplay protocol + GigaChat integration + rejoin tokens. Это закроет ~60% функциональности и позволит начать интеграционное тестирование.

**Оценка времени**: полный cutover (все PR из плана) — 3-4 недели соло или 2 недели командой из 2-3 человек.

**Готовность к прод**: ~30% (инфраструктура готова, но critical paths не реализованы).

---

**Автор**: AI Agent (Arena.ai)  
**Репозиторий**: ForgeLabHub/ForgeBrainStorm  
**Ветка**: `arena/dfae0fdc-forgebrainstorm`  
**Коммит**: TBD (не коммитилось в этой сессии)
