# AGENTS.md — правила для ИИ-агентов в ForgeBrainStorm

Этот файл читают OpenHands и другие агенты до начала работы. Цель правил одна:
агент делает маленькие проверяемые шаги и не оставляет репозиторий в состоянии,
когда «ничего не собирается, а CI красный».

## Карта репозитория

| Путь | Что это | Проверка |
|---|---|---|
| `rust-server/` | основной realtime-бэкенд: axum, tokio, sqlx/PostgreSQL, WebSocket-игровой цикл, провайдеры ИИ | `make agent-verify` |
| `frontend/` | UI на SvelteKit (Svelte 5 runes, Tailwind, adapter-node) | `make frontend-check` |
| `app/`, `templates/`, `static/`, `run.py`, `wsgi.py` | legacy-монолит Flask + Socket.IO на SQLite (отдаётся на Render) | `make lint test` |
| `docs/` | архитектура, API, эксплуатация, безопасность (RU + EN + PDF) | `make docs-check` |

## Железные правила

1. **Атомарность.** Один логический шаг = один-два файла. Не переписывай модуль
   целиком «за один промпт»: так теряется контекст и начинается каскад ошибок.
2. **Компиляция раньше функционала.** После любого изменения в Rust запусти
   `make agent-verify` (fmt → check → clippy `-D warnings` → test). Пока это не
   зелено, новую функциональность не добавляй.
3. **`unwrap()` запрещён** в продакшен-коде (сейчас в `rust-server/src` их ноль).
   Используй `?`, `expect("конкретная причина")` или `AppError`.
4. **Тесты.** Меняешь логику — добавь или обнови юнит-тест в том же файле
   (`#[cfg(test)]`). Тесты, которым нужна живая база, помечай `#[ignore]`.
5. **Стоп после двух попыток.** Если одна и та же ошибка компиляции не уходит
   после двух попыток — `git checkout -- .`, остановись и опиши проблему
   пользователю. Не «лечи» её отключением проверок.
6. **Не читай проект целиком.** Ищи точечно: `rg <символ>` (если ripgrep не
   установлен — `grep -rn`). Читай только найденные файлы и их окружение.
7. **CI остаётся зелёным.** Никаких `#[allow(...)]`, `continue-on-error` и
   правок воркфлоу ради «позеленить». Если проверка мешает — объясни почему.
8. **Git.** В `main` не пушим, PR не открываем без явной просьбы, историю не
   переписываем.

## Команды

| Команда | Что делает |
|---|---|
| `make agent-verify` | гейт для агента: rustfmt, `cargo check --all-targets`, clippy с `-D warnings`, `cargo test` |
| `make agent-verify-all` | то же + ruff, pytest, `svelte-check`, сборка фронтенда |
| `make rust-verify` | только Rust-матрица |
| `make frontend-check` | `svelte-check` (0 ошибок) + `npm run build` |
| `make lint test` | ruff + pytest для Flask-части |
| `make verify` | lint + test + docs-check (локальная матрица Python) |
| `make docs-check` | структура документации, ссылки, PDF |

Перед тем как сказать «готово», приведи вывод команды, а не пересказ.

## Rust: что легко сломать

- Слои: `domain/` без I/O → `services/` → `routes/` и `ws/`. Ошибки — только
  через `AppError`; варианты `Database` и `Config` боксированы специально, чтобы
  `Result` оставался маленьким. Не разбоксовывай без причины.
- Rate limiting лежит за фичей `rate-limit` и **не включён**: за Caddy
  `ConnectInfo` — это адрес прокси, и лимит по нему задушит всех игроков сразу.
  Включать только вместе с ключеванием по `X-Forwarded-For`.
- Конфиг валидируется fail-fast в production (`app/config.py` и
  `rust-server/src/config.rs`): `SECRET_KEY` ≥ 24, `ADMIN_SECRET_KEY` ≥ 12,
  `CHEAT_TESTER_CODE` ≥ 8 символов.

## Frontend: что легко сломать

- Только руны Svelte 5: `$:` в компонентах запрещён (`onMount`, `$derived`,
  `$effect`), `<slot />` запрещён (`let { children } = $props()` + `{@render children()}`).
- `npm run check` должен давать 0 ошибок и 0 предупреждений.
- Ответы WebSocket приходят с полем `correct_index`, игроки — с полем `name`.

## Секреты

- `.env` в `.gitignore` и в репозиторий не попадает. CI блокирует коммит с
  секретом (gitleaks, шаг `No committed secrets`).
- В примерах (`.env.example`, `BUILD.md`) — только плейсхолдеры вида
  `change-me-...`. Allowlist в `.gitleaks.toml` расширяем лишь с объяснением.
