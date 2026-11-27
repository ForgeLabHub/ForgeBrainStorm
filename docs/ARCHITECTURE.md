# Архитектура / Architecture

Карта системы и правила, по которым она развивается. Документ обновляется в том же
PR, который меняет структуру.

- Текущее состояние и путь миграции: [раздел 2](#2-миграция-на-workspace-карта-кода)
- Правила зависимостей между крейтами: [раздел 3](#3-правила-слоёв)
- Решения и их причины: [`docs/adr/`](adr/)

---

## 1. Что есть сегодня

```
┌───────────────────────────┐   /api, /ws   ┌──────────────────────────────┐
│ frontend/ (SvelteKit)     │ ────────────▶ │ crates/server (axum + tokio) │
│ SSR + client, adapter-node│ ◀──────────── │ PostgreSQL через sqlx        │
└───────────────────────────┘   WebSocket   └──────────────────────────────┘
```

- `crates/server` — HTTP + WebSocket сервер, вся игровая логика, доступ к БД.
  Единственный Rust-крейт с кодом на данный момент (~6 000 строк).
- `frontend/` — SvelteKit-приложение (страницы, игровой клиент на WebSocket).
- `app/`, `templates/`, `run.py` — legacy-монолит на Flask. Живёт отдельно,
  покрыт своими тестами, не участвует в Rust-сборке.
- `crates/server/experimental/` — наработки по горизонтальному масштабированию
  (Redis pub/sub, RAG, DDA). Не подключены к сборке; изолируются фичей
  `experimental`, а не удаляются.

## 2. Миграция на workspace: карта кода

Целевая структура описана в issue «Phase 1: workspace skeleton». Таблица ниже —
источник истины о том, куда переезжает каждый модуль. Статус: ✅ сделано,
🚧 в работе, ⏳ запланировано.

| Сейчас | Целевой крейт | Что это | Статус |
| --- | --- | --- | --- |
| `crates/server` (весь крейт) | `crates/server` | перенос без изменения кода | ✅ PR-1.1 |
| — | `crates/protocol/src/messages.rs` | WS-конверт `{type,data}`, типизированные `ClientMessage`/`ServerMessage` вместо `serde_json::Value` | ⏳ PR-1.2 |
| — | `crates/protocol/src/dto.rs` | DTO REST-роутов (`RegisterRequest`, `RoomSnapshot`, …), сейчас разбросаны по `routes/*.rs` | ⏳ PR-1.2 |
| — | `crates/protocol/src/ids.rs` | `RoomCode`, `UserId`, `Qid` — newtype'ы вместо `String`/`i64` | ⏳ PR-1.2 |
| — | `crates/protocol/src/validation.rs` | общие валидаторы (код комнаты, тема, границы настроек) | ⏳ PR-1.2 |
| `src/domain/{room,player,mode,power_up,cheat,role}.rs` | `crates/domain` | чистая логика без I/O | ⏳ PR-1.3 |
| `src/domain/room.rs` (скоринг внутри) | `crates/domain/src/scoring/` | формулы очков по режимам | ⏳ PR-1.3 |
| `src/db.rs`, `src/services/{accounts,history,leaderboard,audit}.rs` | `crates/storage/src/postgres/` | репозитории sqlx | ⏳ PR-2.x |
| `src/services/questions.rs` (кэш) | `crates/storage/src/redis/` | кэш вопросов и снапшоты комнат | ⏳ PR-2.1 |
| `src/services/ai/**` (router, cache, providers) | `crates/ai` | провайдеры, роутер, кэш, quality gate | ⏳ PR-4.x |
| `src/config.rs` (секция `ai_providers`) | `crates/ai` + `crates/server/src/config.rs` | конфиг провайдеров переезжает к ним | ⏳ PR-4.x |
| `src/routes/**`, `src/middleware/**`, `src/ws/**`, `src/error.rs`, `src/metrics.rs`, `src/state.rs` | `crates/server` | HTTP/WS слой, склейка | ✅ остаётся |
| `migrations/` | `migrations/` (корень) | общие для storage и server | ⏳ PR-2.x |
| `config/default.toml` | `crates/server/config/` | конфиг сервера | ✅ |
| `experimental/**` | `crates/server/experimental/` | изолировать фичей `experimental` | ⏳ отдельный PR |
| `tests/health.rs`, `tests/integration_test.rs` | `crates/server/tests/` | интеграционные тесты | ✅ |
| `frontend/` | `crates/leptos-app` | полный переезд на Leptos — Фаза 6 | ⏳ PR-6.x |

### Почему протокол — отдельный крейт

Сегодня WS-протокол существует в двух несогласованных копиях: сервер собирает
JSON через `serde_json::json!`, клиент описывает те же сообщения в TypeScript.
Опечатка в имени поля не ловится ни компилятором, ни тестами. `protocol`
становится единственным описанием формата, компилируется под
`wasm32-unknown-unknown` и покрывается snapshot-тестами (PR-1.2).

## 3. Правила слоёв

```
domain      ← protocol            (только внутрь, никакого I/O)
storage     ← domain, protocol
ai          ← protocol, domain
server      → domain, storage, ai, protocol
leptos-app  → protocol            (+ server_fns, feature = "ssr")
cli         → server, storage     (dev-команды, миграции)
```

Запрещено:

- `domain` зависит от `tokio`/`axum`/`sqlx`/`redis` — иначе логику нельзя
  тестировать без рантайма и переиспользовать в WASM.
- `protocol` зависит от любого крейта workspace — он должен собираться первым и
  отдельно.
- `server` протекает в `storage`/`ai` (например, `storage` не знает про `axum`).

Проверка: `cargo xtask ci` собирает весь workspace, а CI дополнительно собирает
`protocol` под `wasm32-unknown-unknown`.

## 4. Как проверять локально

```bash
cargo xtask ci          # fmt --check + clippy -D warnings + тесты
cargo xtask test        # только тесты
cargo build --workspace # сборка всех крейтов
```

`cargo xtask` — обёртка над теми же командами, что гоняет CI (см. `xtask/src/main.rs`),
чтобы локальная проверка и пайплайн не расходились.

## 5. Известные долги

| Долг | Где | План |
| --- | --- | --- |
| WS-протокол без типов | `crates/server/src/ws/mod.rs` (~1 100 строк) | PR-1.2, затем PR-2.4 (postcard) |
| `unwrap`/`expect` в hot path | `ws/mod.rs`, `middleware/*` | `#![deny(clippy::unwrap_used)]` в `domain` (PR-1.3), затем в остальных |
| Заявленный MSRV 1.85 не собирался | `Cargo.toml` | исправлено на 1.89 в PR-1.1 (см. ADR-0001) |
| Комнаты только в памяти | `ws/mod.rs` | PR-2.1 (Redis snapshots) |
| `experimental/` не собирается | `crates/server/experimental` | изолировать фичей |
