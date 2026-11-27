# Устройство продукта

Карта актуальной системы: `rust-server/` + `frontend/`. Legacy-монолит Flask
(`app/`) описан отдельно в [system-context.md](system-context.md) и в сборку
продукта не входит.

## Три процесса

```
браузер
  │  HTTP  /api/*            WebSocket /ws
  ▼
frontend/ (SvelteKit, adapter-node)
  │  проксирует /api и /ws на бэкенд
  ▼
rust-server/ (axum + tokio + sqlx) :5000
  ├── PostgreSQL :5432   комнаты, пользователи, история, рейтинг
  └── Redis :6379        кэш ответов AI (опционально)
        │
        └── AI-провайдер (openrouter / gigachat / ollama) — опционально,
            без него вопросы берутся из офлайн-банка
```

Локально: `docker compose up -d postgres redis`, затем `cargo run` в
`rust-server` и `npm run dev` в `frontend` ([BUILD.md](../../BUILD.md)).

## Слои бэкенда

| Слой | Каталог | Ответственность | Чего не делает |
| --- | --- | --- | --- |
| Домен | `src/domain/` | `Room`, `Player`, `GameMode`, усиления; правила и очки | не знает про HTTP, БД и сокеты |
| Сервисы | `src/services/` | аккаунты, рейтинг, история, аудит, вопросы, AI-роутер | не формируют ответы HTTP |
| Транспорт | `src/routes/`, `src/ws/` | REST-маршруты и игровой цикл по WebSocket | не считают очки сами — спрашивают домен |
| Инфраструктура | `src/middleware/`, `src/db.rs`, `src/config.rs`, `src/security.rs` | auth, CORS, rate limit, пул соединений, конфиг, хэши | не содержат игровых правил |

Правило, которое стоит держать: **источник истины — домен**. `ws/` и `routes/`
не сравнивают строки режимов и не пересчитывают очки, а вызывают методы
`GameMode` ([game-modes.md](../game-modes.md)). Поэтому новый режим — это один
вариант enum плюс ветки в его методах, а не правки в десятке мест.

## Состояние игры

- Комната живёт в памяти процесса (`domain::Room`) и в PostgreSQL для
  долгоживущих сущностей: пользователи, история партий, рейтинг.
- Таймер, приём ответа и начисление очков — на сервере. Клиент рисует то, что
  пришло в событиях `question_emitted`, `timer_update`, `question_result`,
  `game_over`.
- `game_over` — единственный источник итогов: он приносит `final_leaderboard`,
  `teams` (две стороны в `team`, одна общая в `coop`) и `persisted`
  (`false` у тренировки).

## Фронтенд

| Путь | Экран |
| --- | --- |
| `src/routes/+page.svelte` | лендинг |
| `src/routes/auth/{login,register}` | вход и регистрация |
| `src/routes/lobby` | список комнат |
| `src/routes/room/create` | настройка комнаты |
| `src/routes/room/[code]` | игра: вопрос, результат, итоги |
| `src/routes/settings` | AI-провайдеры |
| `src/routes/profile`, `leaderboard` | профиль и рейтинг |

Общее вынесено в `src/lib/`: `api.ts` (REST), `ws.ts` (сокет с
переподключением), `ui.ts` (подписи режимов и сложностей), `Icon.svelte`
(иконки вместо эмодзи). Подписи режимов живут только в `ui.ts` — дублировать
их в страницах не нужно.

## Конфигурация и секреты

`rust-server/src/config.rs` собирает конфиг из `config/default.toml` и
переменных с префиксом `BRAINSTORM_`, вложенность — через `__`
(`BRAINSTORM__DEFAULT_AI_PROVIDER`). Обязательные ключи (`SECRET_KEY`,
`ADMIN_SECRET_KEY`, `DATABASE_URL`) проверяются на старте, чтобы сервис падал
сразу, а не на первом запросе. Секреты в репозиторий не коммитятся
([security/secrets.md](../security/secrets.md)).

## Что проверяется автоматически

CI (`.github/workflows/ci.yml`) держит шесть ворот: `cargo fmt`/`clippy -D
warnings`/`test`, `svelte-check` + сборка фронтенда, линт и тесты legacy-API,
сборка Docker-образа с проверкой `/health`, отсутствие закоммиченных секретов,
полнота документации (`scripts/docs-check.sh`). Локально то же самое —
`make agent-verify-all` (rust + python + frontend) и `make verify`
([verification/matrix.md](../verification/matrix.md)).
