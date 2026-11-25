# ПРОМТ ДЛЯ AI-АГЕНТА: Проверка и компиляция Rust-бэкенда BrainStorm

## 📋 КОНТЕКСТ

Ты работаешь с проектом BrainStorm — платформой интерактивных викторин.

**Репозиторий**: `ForgeLabHub/ForgeBrainStorm`
**Язык бэкенда**: Rust (Axum + PostgreSQL + Tokio)
**Расположение кода**: `rust-server/`
**Статус**: Код написан, но **никогда не компилировался**. Ожидается много ошибок.

**Что есть**:
- `rust-server/Cargo.toml` — зависимости
- `rust-server/src/main.rs` — точка входа
- `rust-server/src/config.rs` — конфигурация (figment)
- `rust-server/src/db.rs` — PostgreSQL pool (sqlx)
- `rust-server/src/error.rs` — ошибки (thiserror)
- `rust-server/src/domain/` — Room, Player модели
- `rust-server/src/services/` — business logic, включая multi-AI
- `rust-server/src/services/ai/` — multi-AI провайдеры (GigaChat, OpenAI, Ollama)
- `rust-server/src/routes/` — HTTP handlers
- `rust-server/src/middleware/` — auth, CORS, rate-limit
- `rust-server/src/ws/` — WebSocket handler
- `rust-server/src/security.rs` — HMAC, утилиты
- `rust-server/migrations/` — SQL схема
- `rust-server/config/default.toml` — конфиг
- `rust-server/Dockerfile` + `docker-compose.yml`

**Что НЕ сделано**:
- Код не проверен компилятором (`cargo check`)
- Типы не проверены
- Runtime behavior не проверен
- Интеграционные тесты не написаны
- Frontend (`frontend/`) тоже не собирался

---

## 🎯 ТВОЯ ЗАДАЧА

**Цель**: Довести проект до состояния, когда всё компилируется и базовые тесты проходят.

**Не добавляй новых фич** — только исправляй ошибки компиляции и warnings.

---

## 📝 ПЛАН ДЕЙСТВИЙ (СТРОГО ПО ПОРЯДКУ)

### Этап 1: Проверка Rust-кода

```bash
cd rust-server

# 1.1. Проверь синтаксис и типы
cargo check 2>&1 | tee /tmp/check1.log

# 1.2. Если есть ошибки — исправь их ОДНУ ЗА ДРУГОЙ
# После каждых 3-5 исправлений запускай cargo check снова

# 1.3. Когда cargo check проходит — запусти clippy
cargo clippy -- -D warnings 2>&1 | tee /tmp/clippy.log

# 1.4. Исправь все clippy warnings

# 1.5. Проверь форматирование
cargo fmt --check

# 1.6. Если форматирование неверное — запусти cargo fmt
cargo fmt
```

### Этап 2: Сборка

```bash
# 2.1. Собери debug-версию
cargo build

# 2.2. Собери release-версию
cargo build --release

# 2.3. Запусти тесты (если есть)
cargo test
```

### Этап 3: Проверка Docker

```bash
# 3.1. Собери Docker-образ
docker build -t brainstorm:test .

# 3.2. Запусти docker compose
docker compose up -d postgres redis

# 3.3. Запусти приложение (либо через cargo, либо через Docker)
# Вариант A: через cargo
cargo run

# Вариант B: через docker compose
docker compose up app

# 3.4. Проверь health endpoints
curl http://localhost:5000/health
# Ожидаемый ответ: {"status":"ok","version":"0.1.0"}

curl http://localhost:5000/readyz
# Ожидаемый ответ: {"status":"ready","db":"ok"}
```

### Этап 4: Проверка Frontend

```bash
cd ../frontend

# 4.1. Установи зависимости
npm install

# 4.2. Проверь типы
npm run check

# 4.3. Исправь ошибки типов если есть

# 4.4. Собери проект
npm run build
```

### Этап 5: Коммит и пуш

```bash
cd ..

# 5.1. Добавь все изменения
git add -A

# 5.2. Закоммить с понятным сообщением
git commit -m "fix: resolve compilation errors and clippy warnings

- Fixed all cargo check errors
- Resolved all clippy warnings
- Applied cargo fmt
- Verified docker build
- Verified /health and /readyz endpoints
- Verified frontend build"

# 5.3. Запушь в ветку
git push
```

---

## 🚨 ОЖИДАЕМЫЕ ПРОБЛЕМЫ (и как их решать)

### 1. Несовпадение версий crate'ов

**Симптом**: Ошибки типа `method not found`, `trait not implemented`.
**Решение**: Проверь `Cargo.toml`, обнови версии если нужно. Возможно, нужно запустить `cargo update`.

### 2. Неправильные импорты

**Симптом**: `unresolved import`, `could not find X in Y`.
**Решение**: Проверь структуру модулей, добавь недостающие `pub use`, исправь пути импортов.

### 3. `async_trait` проблемы

**Симптом**: Ошибки с async fn в traits.
**Решение**: Убедись, что все traits с async методами помечены `#[async_trait]`, и что `async-trait` добавлен в `Cargo.toml`.

### 4. parking_lot::RwLock через .await

**Симптом**: `future cannot be sent between threads safely`.
**Решение**: Найди все места, где `RwLockReadGuard` или `RwLockWriteGuard` переживает `.await`. Замени `parking_lot::RwLock` на `tokio::sync::RwLock` в этих местах, или переструктурируй код так, чтобы lock отпускался до `.await`.

```rust
// ПЛОХО — lock переживает .await
let data = self.registry.read().await;
some_async_call().await;
drop(data);

// ХОРОШО — lock отпускается до .await
let data = {
    let guard = self.registry.read().await;
    guard.clone() // или скопируй нужные данные
};
some_async_call().await;
```

### 5. sqlx compile-time checks

**Симптом**: Если используются `sqlx::query!` — нужен `DATABASE_URL`.
**Решение**: В текущем коде используются `sqlx::query` (runtime checks), что должно работать. Но если есть `query!` — замени на `query` или настрой `DATABASE_URL`.

### 6. WebSocket auth middleware

**Симптом**: Типы не совпадают в `ws_auth_middleware`.
**Решение**: Проверь сигнатуры:
- `State<SessionCache>` должен быть в `AppState`
- `axum::middleware::from_fn_with_state` должен принимать правильные типы
- Cookie extraction может требовать `axum_extra::extract::CookieJar`

### 7. Missing `async-trait` в Cargo.toml

**Симптом**: `cannot find attribute async_trait in this scope`.
**Решение**: Добавь в `Cargo.toml`:
```toml
async-trait = "0.1"
```

### 8. Missing `chrono` в некоторых файлах

**Симптом**: `chrono::DateTime` не найден.
**Решение**: Добавь `use chrono;` или `use chrono::{DateTime, Utc};` где нужно.

### 9. ProviderConfig не совпадает с AiProviderConfig

**Симптом**: Типы не совпадают между `config.rs` и `services/ai/`.
**Решение**: Унифицируй структуры. В `config.rs` есть `AiProviderConfig`, в `services/ai/mod.rs` — `ProviderConfig`. Создай конвертацию между ними.

### 10. DashMap в async контексте

**Симптом**: `DashMap` не `Send` + `Sync` в некоторых местах.
**Решение**: `DashMap` уже `Send + Sync`, но проверь, что ты не держишь `Ref` через `.await`.

### 11. broadcast::channel capacity

**Симптом**: `broadcast::channel` может требовать `capacity > 0`.
**Решение**: Проверь, что capacity > 0 (сейчас 256 — ок).

### 12. Missing `uuid` в Cargo.toml

**Симптом**: `uuid::Uuid` not found.
**Решение**: Убедись, что в `Cargo.toml` есть:
```toml
uuid = { version = "1", features = ["v4"] }
```

---

## ✅ КРИТЕРИИ УСПЕХА

Проект считается готовым, когда:

1. ✅ `cargo check` проходит без ошибок
2. ✅ `cargo clippy -- -D warnings` проходит без warnings
3. ✅ `cargo fmt --check` проходит (или запущен `cargo fmt`)
4. ✅ `cargo build` успешно собирает бинарник
5. ✅ `cargo build --release` успешно собирает release-версию
6. ✅ `docker build -t brainstorm:test .` успешно собирает образ
7. ✅ `docker compose up -d postgres redis` запускает БД
8. ✅ `cargo run` запускает сервер без panic
9. ✅ `curl http://localhost:5000/health` возвращает `{"status":"ok",...}`
10. ✅ `curl http://localhost:5000/readyz` возвращает `{"status":"ready",...}`
11. ✅ `cd frontend && npm install && npm run check` проходит без ошибок
12. ✅ `cd frontend && npm run build` успешно собирает frontend
13. ✅ Все изменения закоммичены и запушены

---

## 🚫 АНТИ-ПАТТЕРНЫ (ЧЕГО НЕ ДЕЛАТЬ)

1. ❌ **Не добавляй новых фич** — только исправляй ошибки
2. ❌ **Не переписывай архитектуру** — исправляй то, что есть
3. ❌ **Не используй `unwrap()` в проде** — только `?` или `expect()` с понятным сообщением
4. ❌ **Не игнорируй clippy warnings** — исправь все
5. ❌ **Не коммить без проверки** — запусти `cargo check` перед коммитом
6. ❌ **Не удаляй существующий код без причины** — только если он точно не нужен
7. ❌ **Не меняй API контракты** — endpoints должны остаться теми же
8. ❌ **Не меняй схему БД** — миграции уже написаны
9. ❌ **Не добавляй новые зависимости без необходимости** — сначала попробуй исправить с тем, что есть
10. ❌ **Не оставляй `TODO` без комментариев** — если не можешь исправить сразу, напиши подробный комментарий

---

## 📊 ОТЧЁТ О РЕЗУЛЬТАТАХ

После завершения работы создай файл `COMPILATION_REPORT.md` со следующей структурой:

```markdown
# Отчёт о компиляции

## Статистика
- Время работы: X минут
- Исправлено ошибок: N
- Изменено файлов: M
- Добавлено строк: +X
- Удалено строк: -Y

## Исправленные проблемы
1. [описание проблемы] → [как исправлено]
2. ...

## Оставшиеся issues
1. [описание] — [почему не исправлено]
2. ...

## Проверки
- [x] cargo check
- [x] cargo clippy
- [x] cargo fmt
- [x] cargo build
- [x] docker build
- [x] /health endpoint
- [x] /readyz endpoint
- [x] frontend build

## Заметки
[Любые дополнительные заметки о коде, архитектуре, etc.]
```

---

## 💡 СОВЕТЫ

1. **Работай итеративно**: исправь 3-5 ошибок → `cargo check` → ещё 3-5 → `cargo check`. Не пытайся исправить всё сразу.

2. **Читай ошибки внимательно**: Rust compiler даёт отличные сообщения об ошибках. Часто он сам предлагает исправление.

3. **Используй `cargo check` чаще, чем `cargo build`**: `check` быстрее, не создаёт бинарник.

4. **Коммить часто**: после каждых 5-10 исправлений делай коммит. Это позволит откатиться если что-то пойдёт не так.

5. **Не бойся удалять мёртвый код**: если какой-то код не используется и вызывает ошибки — удали его.

6. **Проверяй документацию**: если не понимаешь как использовать crate — зайди на docs.rs и почитай.

7. **Используй `rust-analyzer`**: если ты в VS Code — установи rust-analyzer, он подскажет ошибки в реальном времени.

8. **Если застрял — гугли**: ошибки Rust очень хорошо гуглятся. 99% проблем уже кто-то решил на StackOverflow.

---

## 🎯 ПРИОРИТЕТЫ

Если не можешь исправить всё — сфокусируйся на этом порядке:

1. **Критично**: `cargo check` должен проходить (иначе проект мёртв)
2. **Важно**: `cargo clippy` должен проходить (иначе код некачественный)
3. **Желательно**: Docker build должен работать (иначе нельзя деплоить)
4. **Опционально**: Frontend build (можно починить позже)
5. **Nice-to-have**: Интеграционные тесты (можно добавить в следующем PR)

---

## 📚 ПОЛЕЗНЫЕ ССЫЛКИ

- Rust Book: https://doc.rust-lang.org/book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/
- Axum docs: https://docs.rs/axum/latest/axum/
- SQLx docs: https://docs.rs/sqlx/latest/sqlx/
- Tokio docs: https://docs.rs/tokio/latest/tokio/
- Serde docs: https://docs.rs/serde/latest/serde/

---

## 🏁 НАЧАЛИ

Готов? Начинай с `cargo check` в `rust-server/`.

Удачи! 🚀
