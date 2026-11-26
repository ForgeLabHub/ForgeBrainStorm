# Чеклист для локальной проверки

**Важно**: Код в этом PR **не компилировался** в sandbox (нет Rust toolchain). Перед мержем необходимо выполнить следующие проверки локально.

## 1. Компиляция Rust backend

```bash
cd rust-server
cargo check
```

**Ожидаемые проблемы** (исправить до мержа):
- [ ] Возможно, неправильные импорты в `main.rs` (добавлен `ws_auth_middleware`)
- [ ] Возможно, проблемы с типами в `SessionCache::get_or_load`
- [ ] Возможно, проблемы с `axum::middleware::from_fn_with_state`

Если есть ошибки компиляции — исправить и закоммитить.

## 2. Сборка release

```bash
cargo build --release
```

**Ожидаемое время**: 5-10 минут на первой сборке.

## 3. Clippy и форматирование

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

Исправить все warnings.

## 4. Frontend проверка

```bash
cd frontend
npm install
npm run check
```

**Ожидаемые проблемы**:
- [ ] Возможно, отсутствуют зависимости в `package.json`
- [ ] Возможно, проблемы с типами в `api.ts` или `ws.ts`

## 5. Docker build

```bash
cd rust-server
docker build -t brainstorm:test .
```

**Размер образа**: Ожидается 50-80 MB (не 15-20 MB, как написано в README — это было оптимистично).

## 6. Запуск сервера

```bash
docker compose up -d postgres redis
cargo run
```

Или:

```bash
docker compose up --build
```

## 7. Проверка /health endpoint

```bash
curl http://localhost:5000/health
```

**Ожидаемый ответ**:
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

## 8. Проверка /readyz endpoint

```bash
curl http://localhost:5000/readyz
```

**Ожидаемый ответ** (если Postgres работает):
```json
{
  "status": "ready",
  "db": "ok"
}
```

## 9. Проверка WebSocket auth

```bash
# Без cookie — должен вернуть 401
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: $(openssl rand -base64 16)" http://localhost:5000/ws

# С валидной сессией — должен upgrade
# (нужно сначала получить сессию через /api/auth/login)
```

## 10. Проверка миграций

```bash
# Подключиться к Postgres
docker compose exec postgres psql -U brainstorm

# Проверить таблицы
\dt

# Ожидаемые таблицы:
# users, sessions, bans, audit_log, leaderboard, leaderboard_ranked (view),
# achievements, campaign_progress, room_history, ugc_questions, question_cache
```

## Критические архитектурные проверки

### A. Parking_lot::RwLock через .await

Проверить, что нет `.await` внутри критических секций `RwLock`:

```bash
# Найти все места, где используется rooms.write() или rooms.read()
grep -rn "rooms\.write()" rust-server/src/
grep -rn "rooms\.read()" rust-server/src/

# Для каждого найденного места проверить, что внутри нет .await
# Если есть — заменить на tokio::sync::RwLock
```

**Статус**: Проверено вручную — критических проблем не найдено. Но требует автоматической проверки.

### B. Broadcast channel per-room

Проверить, что broadcast channel создаётся per-room, а не глобально:

```bash
grep -n "broadcast::channel" rust-server/src/domain/room.rs
```

**Ожидаемый результат**: Строка 108 в `Room::new()` — создаётся per-room. ✅

### C. WS auth middleware

Проверить, что WS handler требует авторизацию:

```bash
grep -A5 "route(\"/ws\"" rust-server/src/main.rs
```

**Ожидаемый результат**: Должен быть `.layer(axum::middleware::from_fn_with_state(..., ws_auth_middleware))`. ✅ (добавлено)

### D. sqlx query! vs query

Проверить, используются ли compile-time checked queries:

```bash
grep -n "sqlx::query!" rust-server/src/**/*.rs
```

**Ожидаемый результат**: Пусто — все queries runtime-checked. Это проблема для production, но приемлемо для первого PR.

**Рекомендация**: Мигрировать на `query!` позже, когда будет CI с `DATABASE_URL`.

### E. Argon2 params

Проверить параметры хэширования:

```bash
grep -n "Argon2::default()" rust-server/src/routes/auth.rs
```

**Ожидаемый результат**: Строки 54 и 101. Это дефолтные параметры (m=19456 KB, t=2, p=1), что может быть медленно при высокой нагрузке.

**Рекомендация**: Для production использовать кастомные параметры:
```rust
let params = argon2::Params::new(19456, 2, 1, None).unwrap();
let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, params);
```

## Что делать, если что-то не работает

### Ошибки компиляции

1. Прочитать сообщение об ошибке
2. Исправить проблему
3. Закоммитить: `git commit -am "fix: <описание>"`
4. Запушить: `git push origin arena/dfae0fdc-forgebrainstorm`

### Ошибки runtime

1. Проверить логи: `docker compose logs app`
2. Проверить, что Postgres и Redis работают: `docker compose ps`
3. Проверить переменные окружения в `docker-compose.yml`

### Проблемы с миграциями

Если миграции не применяются:

```bash
# Удалить volume
docker compose down -v

# Пересоздать
docker compose up -d postgres
docker compose up app
```

## После успешной проверки

1. Закоммитить все исправления
2. Запушить в ветку
3. Обновить PR с результатами проверки
4. Добавить в описание PR:
   - `cargo check` — ✅
   - `cargo clippy` — ✅
   - `cargo build --release` — ✅
   - Docker build — ✅ (размер: X MB)
   - `/health` — ✅
   - `/readyz` — ✅
   - WS auth — ✅

## Если не получается исправить

Создать issue с описанием проблемы и приложить:
- Версию Rust: `rustc --version`
- Версию cargo: `cargo --version`
- Полный вывод `cargo check 2>&1`
- OS: `uname -a`

## Честный статус

**До выполнения этого чеклиста**: 0% проверено, 30% написано.

**После выполнения**: Можно говорить о реальной готовности.

---

**Обновлено**: 2026-11-25  
**Автор**: AI Agent (Arena.ai)  
**Статус**: Требуется локальная проверка
