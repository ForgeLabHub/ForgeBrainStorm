# 🧠 BrainStorm — Мозговой Штурм

**Realtime AI quiz platform: pick a topic, the model writes the questions, friends join by code and play.**
**Платформа викторин в реальном времени: тема — любая, вопросы пишет ИИ, друзья подключаются по коду комнаты.**

<p>
  <a href="docs/index.md"><img alt="docs" src="https://img.shields.io/badge/docs-полная%20документация-blue"></a>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.85%2B-000000?logo=rust">
  <img alt="SvelteKit" src="https://img.shields.io/badge/SvelteKit-5-FF3E00?logo=svelte">
  <img alt="PostgreSQL" src="https://img.shields.io/badge/PostgreSQL-16%2B-4169E1?logo=postgresql">
  <a href="LICENSE"><img alt="license" src="https://img.shields.io/badge/license-MIT-green"></a>
</p>

---

## Что это / What it is

Хост выбирает тему, сложность и режим — бэкенд генерирует вопросы через ИИ
(OpenRouter по умолчанию, офлайн-банк как резерв), создаёт комнату с
шестизначным кодом. Игроки заходят по коду, отвечают одновременно, очки идут в
рейтинг, XP и историю игр.

The host picks a topic, difficulty and mode; the backend asks an LLM for the
questions (OpenRouter by default, offline bank as fallback) and opens a room with
a six-character code. Players join, answer in parallel, and results land in the
leaderboard, XP and game history.

## Режимы игры / Game modes

| Режим | Код | Правила |
|---|---|---|
| 🎯 Классика | `classic` | Верный ответ — 100 очков, без жизней |
| ⚔️ Каждый за себя | `ffa` | Очки зависят от скорости: 40 на последней секунде … 100 мгновенно |
| ❤️ На выживание | `lives` | 3 жизни, ошибка отнимает одну, последний выживший заканчивает раунд |
| ⚡ Блиц | `blitz` | Таймер фиксирован на 10 с, серия верных ответов даёт до ×2 очков |
| 👥 Команды | `team` | Игроки рассаживаются в «Синие» и «Красные», побеждает сумма очков команды |
| 🎓 Тренировка | `sandbox` | Без жизней и без записи в рейтинг — просто попробовать |

Полные правила, формулы очков и краевые случаи — [docs/game-modes.md](docs/game-modes.md).

## Возможности / Features

- **Комнаты в реальном времени** — WebSocket, общий таймер, мгновенные результаты и таблица лидеров.
- **Шесть режимов** — от классики до командного блица ([docs/game-modes.md](docs/game-modes.md)).
- **Усиления** — двойные очки, заморозка таймера, удаление двух вариантов; рука приходит только владельцу.
- **ИИ-провайдеры** — маршрутизатор с приоритетами, переключение на ходу (`/api/ai/providers/switch`), кэш вопросов.
- **Аккаунты и прогресс** — регистрация, уровни по XP, монеты, история игр, рейтинг.
- **Админка** — комнаты, пользователи, баны, сброс статистики, аудит-лог.
- **Тёмный «glass» UI** — Svelte 5 runes, Tailwind, адаптив от телефона до проектора.

## Архитектура / Architecture

```
frontend/ (SvelteKit, adapter-node)          rust-server/ (axum + tokio)
┌───────────────────────────┐   /api, /ws   ┌──────────────────────────────┐
│ /                landing   │ ───────────▶ │ routes/   HTTP (auth, rooms, │
│ /auth/login|register       │              │           ai, leaderboard…)   │
│ /lobby  /room/create       │              │ ws/       игровой цикл       │
│ /room/[code]  /profile     │ ◀─────────── │ domain/   правила без I/O    │
│ /settings  /leaderboard    │   события    │ services/ AI, БД, кэш        │
└───────────────────────────┘              └──────────────┬───────────────┘
                                                          │ sqlx
                                                   PostgreSQL 16+
```

- Слои и правила зависимостей: [docs/rust-backend.md](docs/rust-backend.md).
- Модули: [docs/modules/](docs/modules/), контекст системы: [docs/architecture/system-context.md](docs/architecture/system-context.md).
- Схема БД: [docs/database/storage.md](docs/database/storage.md).

## Быстрый старт / Quick start

Нужны: Rust 1.85+, Node 20+, PostgreSQL 16+.

```bash
# 1. База
createdb brainstorm

# 2. Бэкенд (миграции применяются при старте)
cd rust-server
BRAINSTORM_DATABASE_URL=postgres://brainstorm:brainstorm@localhost:5432/brainstorm \
OPENROUTER_API_KEY=sk-or-... \
cargo run                      # http://localhost:5000

# 3. Фронтенд
cd ../frontend
npm install
npm run dev                    # http://localhost:5173, /api и /ws проксируются на :5000
```

Без ключа ИИ приложение остаётся работоспособным: маршрутизатор уходит на
офлайн-банк вопросов, партии идут, в логах будет `no provider available`.

Проверка, что всё живо:

```bash
curl -s localhost:5000/health                 # {"status":"ok",...}
(cd frontend && npx svelte-check --tsconfig ./tsconfig.json)
(cd rust-server && cargo test)
python scripts/e2e-modes.py                   # партии по WS: режимы, команды, блиц, тренировка
```

### Docker

```bash
docker compose -f deploy/docker-compose.prod.yml up -d --build   # Postgres + backend + frontend + Caddy
```

Локальный стек с БД и Redis: `cd rust-server && docker compose up`. Переменные
и требования — [BUILD.md](BUILD.md), продакшен — [docs/operations/deployment.md](docs/operations/deployment.md).

## Переменные окружения / Environment

| Переменная | Назначение | По умолчанию |
|---|---|---|
| `BRAINSTORM_DATABASE_URL` | строка подключения PostgreSQL | `postgres://brainstorm:brainstorm@localhost:5432/brainstorm` |
| `BRAINSTORM_SECRET_KEY` | подпись сессий, ≥ 32 байт в продакшене | dev-значение из `rust-server/config/default.toml` |
| `BRAINSTORM_ADMIN_SECRET_KEY` | ключ активации админки | пусто (админка выключена) |
| `BRAINSTORM_LISTEN` | адрес прослушивания | `0.0.0.0:5000` |
| `BRAINSTORM_CORS_ORIGINS` | JSON-массив разрешённых origin | `["http://localhost:5173"]` |
| `BRAINSTORM_SESSION_COOKIE_SECURE` | cookie только по HTTPS | `false` |
| `OPENROUTER_API_KEY` | ключ OpenRouter (провайдер по умолчанию) | — |
| `OPENAI_API_KEY`, `GIGACHAT_CREDENTIALS` | альтернативные провайдеры | — |
| `RUST_LOG` | фильтр логов tracing | `brainstorm=info` |

Все значения из `rust-server/config/default.toml` переопределяются префиксом
`BRAINSTORM_`. Секреты в репозиторий не попадают: см. [docs/security/secrets.md](docs/security/secrets.md).

## API и WebSocket

| Метод | Путь | Что делает |
|---|---|---|
| `POST` | `/api/auth/register`, `/api/auth/login`, `/api/auth/logout` | аккаунт и сессия (cookie `brainstorm_session`) |
| `GET` | `/api/auth/me` | профиль: XP, уровень, монеты, игры, очки |
| `GET` | `/api/rooms` | живые комнаты |
| `POST` | `/api/rooms` | создать комнату (тема, режим, сложность, число вопросов) |
| `GET` | `/api/rooms/:code` | снимок комнаты |
| `GET` | `/api/leaderboard/top`, `/api/leaderboard/rank/:username` | рейтинг |
| `GET` | `/api/profile/games` | история игр |
| `POST` | `/api/ai/generate`, `GET /api/ai/providers`, `POST /api/ai/providers/switch`, `GET /api/ai/cache/stats` | генерация и провайдеры |
| `GET`/`POST` | `/api/admin/*` | админка (нужен активированный ключ) |

WebSocket `/ws`, сообщения `{ "type": ..., "data": {...} }`:

| Клиент → сервер | Сервер → клиент |
|---|---|
| `join_room {code}` | `room_joined`, `players_updated`, `player_joined`, `player_left` |
| `start_game {}` (хост) | `game_preparing`, `game_started`, `question_emitted` |
| `submit_answer {answer_index}` | `question_result`, `eliminated`, `game_over` |
| `use_power_up {power_up}` | `power_ups`, `options_hidden`, `power_up_used`, `timer_update` |

Подробности протокола — [docs/api/conventions.md](docs/api/conventions.md) и [docs/modules/realtime.md](docs/modules/realtime.md).

## Тесты и качество / Tests

```bash
make agent-verify             # rustfmt --check, cargo check, clippy -D warnings, cargo test
make frontend-check           # svelte-check (0 ошибок) + production build
make docs-check               # обязательные документы, ссылки, полнота спецификаций
python scripts/e2e-modes.py   # живая проверка режимов через HTTP + WebSocket
```

Что именно доказано каким тестом — [docs/verification/matrix.md](docs/verification/matrix.md).

## Структура репозитория

| Путь | Что это |
|---|---|
| `rust-server/` | бэкенд: axum, tokio, sqlx, WebSocket-игровой цикл, ИИ-провайдеры, миграции |
| `frontend/` | SvelteKit-приложение (Svelte 5 runes, Tailwind) |
| `deploy/` | production-стек: PostgreSQL + backend + frontend + Caddy |
| `docs/` | архитектура, API, БД, эксплуатация, безопасность, верификация |
| `scripts/` | `docs-check`, `ws-check`, `e2e-modes`, вспомогательные скрипты |
| `app/`, `templates/`, `static/`, `run.py` | **legacy**: старый Flask + Socket.IO монолит, оставлен для прежнего деплоя |

## Документация

- [docs/index.md](docs/index.md) — карта документов.
- [docs/rust-backend.md](docs/rust-backend.md) — слои, домен, расширение.
- [docs/game-modes.md](docs/game-modes.md) — режимы и формулы очков.
- [BUILD.md](BUILD.md) — сборка с нуля, [DEPLOY.md](DEPLOY.md) — развёртывание.
- [ROADMAP.md](ROADMAP.md) — планы, [CHANGELOG.md](CHANGELOG.md) — история изменений.
- [CONTRIBUTING.md](CONTRIBUTING.md) — правила контрибуции, [AGENTS.md](AGENTS.md) — правила для ИИ-агентов.

## Лицензия

MIT — см. [LICENSE](LICENSE).
