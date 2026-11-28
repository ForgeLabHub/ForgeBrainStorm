# ПРОМТ ДЛЯ НОВОЙ КОНВЕРСАЦИИ: ForgeBrainStorm (состояние после PR #35)

> Скопируй весь текст ниже в новую конверсацию как первое сообщение.
> Значения секретов здесь **сознательно отсутствуют** — они уже инжектятся в новое
> окружение как переменные среды. В промте есть только их имена и правила обращения.

---

Ты продолжаешь работу над **ForgeBrainStorm** — realtime-платформой викторин.
Отвечай по-русски. Сначала прочитай `AGENTS.md` в корне репозитория и следуй его
«железным правилам» (атомарность, компиляция раньше функционала, проверка прогоном).

## 1. Репозиторий и стек

| Путь | Что это | Гейт |
|---|---|---|
| `crates/server/` | основной бэкенд: axum + tokio + sqlx/PostgreSQL + WebSocket-игровой цикл, провайдеры ИИ (крейт `brainstorm`) | `cargo xtask ci` |
| `frontend/` | SvelteKit, Svelte 5 runes, Tailwind, adapter-node | `cd frontend && npm run check` |
| `app/`, `templates/`, `static/`, `run.py`, `wsgi.py` | legacy Flask + Socket.IO на SQLite (крутится на Render) — **не ломать** | `make lint test` |
| `tools/smoke_chat_cheat.py` | живой e2e-прогон по HTTP + WebSocket | см. §5 |
| `docs/`, `CHANGELOG.md` | документация и журнал изменений (RU) | `make docs-check` |

Ветка: `feat/cheat-chat-supabase`, PR **#35** открыт:
https://github.com/ForgeLabHub/ForgeBrainStorm/pull/35
Обновляй именно этот PR (новые коммиты в ту же ветку), новый PR не создавай.

## 2. Секреты: как обращаться (важно)

Ключи уже есть в окружении новой конверсации. Правила:

- Читай только **имена** переменных, значения никогда не печатай, не логируй,
  не пиши в файлы, не коммить, не вставляй в промт/чат/PR.
- Проверить наличие, не раскрывая значение: `test -n "$OPENROUTER_API_KEY" && echo set`.
  Список имён без значений: `env | grep -iE 'openrouter|openai|gigachat|database_url|secret_key' | cut -d= -f1`.
- В коде и конфигах ключ берётся **только** из env (см. `crates/server/src/config.rs`,
  `fill_provider_key`), в `crates/server/config/default.toml` поле `api_key` остаётся пустым.
- Если ключа нет — сервер не падает: роутер пишет `skipping provider: no API key configured`
  и работает офлайн-банк вопросов. Поэтому «игра запустилась» ≠ «ИИ работает».

| Переменная | Куда попадает |
|---|---|
| `OPENROUTER_API_KEY` | провайдер `openrouter` (по умолчанию `default_ai_provider = "openrouter"`) |
| `OPENAI_API_KEY` | провайдер `openai` |
| `GIGACHAT_CREDENTIALS` | провайдер `gigachat` (base64 `client_id:client_secret`) |
| `DATABASE_URL` | Postgres (локально `postgres://brainstorm:brainstorm@localhost:5432/brainstorm`, контейнер `bs-pg`) |
| `SECRET_KEY`, `REDIS_URL`, `PORT` | сессии/HMAC, кэш, порт (по умолчанию 5000) |

OpenRouter говорит по протоколу OpenAI chat-completions и переиспользует
`crates/server/src/services/ai/providers/openai.rs` (`ProviderType::OpenRouter`).

## 3. Что уже сделано (не переделывать, только расширять)

**Чат и эмодзи.** `crates/server/src/domain/chat.rs`: `ChatLog` (последние 50 сообщений,
троттлинг 400 мс, санитайз), палитра эмодзи `EMOJI_PALETTE`, продублирована в
`frontend/src/lib/chat.ts` — при изменении правь обе стороны.

**12 читерских способностей** (`domain/cheat.rs` → `domain/room.rs` → `ws/mod.rs` →
`frontend/src/lib/components/CheatPanel.svelte`):

`peek`, `freeze`, `steal`, `double`, `mute`, `jinx`, `hijack`, `rewrite`,
`leech` (25 очков с каждого), `blind` (жертва не отвечает 10 с), `swap` (обмен очками с
лидером), `hint` (2 неверных варианта скрыты только читеру).

Бюджет живёт в `Player::cheat`, списывается на сервере, приходит приватным событием
`cheat_state`. Пустое применение возвращает использование и отвечает `error` с кодом
`cheat_no_effect`. `blind` проверяется в `submit_answer` (код `blinded`).

**Админ-панель** (`crates/server/src/routes/admin.rs` + `frontend/src/routes/admin/+page.svelte`):

```
GET  /api/admin/stats                    счётчики, в т.ч. live_sessions
GET  /api/admin/users?q&role&limit&offset
POST /api/admin/users/role|reset|delete
GET  /api/admin/bans   POST /api/admin/ban|unban
GET  /api/admin/rooms  GET /api/admin/rooms/detail?code=
POST /api/admin/rooms/kick|announce|chat_lock|chat_clear|close
GET  /api/admin/sessions  POST /api/admin/sessions/revoke
POST /api/admin/broadcast
GET  /api/admin/audit_log?action&limit&offset
GET  /api/admin/ai/status  POST /api/admin/ai/switch|ai/clear_cache
```

**Безопасность сессий.** `auth_middleware` проверяет активный бан в том же запросе, что и
сессию; `POST /ban` отзывает все сессии цели (`revoke_sessions`) и выкидывает её из живых
комнат (`kick_from_rooms`). Смена роли и удаление аккаунта сбрасывают `SessionCache`
(`invalidate_user`). Перезагрузка страницы не снимает мут/проклятие/ослепление — при входе
приходит событие `status`.

Документация обновлена: `CHANGELOG.md` (секция Unreleased) и `docs/rust-backend.md`
(таблицы способностей и админ-эндпоинтов).

## 4. Как запускать

```bash
# Postgres (если не поднят)
docker start bs-pg || docker run -d --name bs-pg -p 5432:5432 \
  -e POSTGRES_USER=brainstorm -e POSTGRES_PASSWORD=brainstorm -e POSTGRES_DB=brainstorm postgres:16

# Бэкенд
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p brainstorm
mkdir -p .run
(nohup ./target/debug/brainstorm > .run/backend.log 2>&1 & echo $! > .run/backend.pid)
curl -s -o /dev/null -w '%{http_code}\n' localhost:5000/health   # ждём 200

# Фронтенд (порт 12000 проксируется наружу)
cd frontend && (nohup npm run dev -- --port 12000 --host 0.0.0.0 > ../.run/frontend.log 2>&1 & echo $! > ../.run/frontend.pid)
```

Перезапуск после сборки — **по pid-файлу**, не через `pkill -f` (в контейнере живут другие
процессы): `kill $(cat .run/backend.pid)`.

Наружу смотрят хосты вида `https://work-1-<id>.prod-runtime.all-hands.dev/` (порт 12000).
Логин админа: любой аккаунт, зарегистрированный с кодом вместо пароля `19112009`;
читер — `1778`.

## 5. Проверка (обязательна перед любым «готово»)

```bash
cargo fmt --all                 # иначе fmt-check в гейте упадёт
cargo xtask ci                  # fmt-check + clippy -D warnings + tests
cd frontend && npm run check    # svelte-check: ждём 0 errors / 0 warnings
python3 tools/smoke_chat_cheat.py http://localhost:5000   # нужен websockets + requests
```

Смоук-тест играет настоящую игру двумя сокетами и проверяет: чат, флуд-контроль, палитру
эмодзи, все способности читера (включая возврат бюджета), заморозку чата, кик, бан
(старая сессия должна получить 401/403), разбан. Успех — строка `ALL CHECKS PASSED`.

Правило: **сначала прогон, потом утверждение.** Без вывода команд не пиши «работает».

## 6. Грабли, на которые уже наступали

1. **`sessions` не имеет колонки `id`** — PK это `token`. Для админских списков выводи
   псевдо-id (`left(md5(token), 12)`) и никогда не отдавай сам токен.
2. **`bans.identifier` для `kind='user'` хранит id пользователя строкой.** Бан/разбан по
   нику без резолва в id молча не совпадает ни с чем (`resolve_user_id`).
3. **sqlx здесь не проверяет SQL на компиляции** (используется runtime-API
   `sqlx::query_as::<_, (…)>`), поэтому опечатка в колонке всплывает только в рантайме —
   после правки запроса обязательно дёрни эндпоинт.
4. **`frontend/**/*.svelte` смешивает табы и пробелы.** Правки «точным совпадением строки»
   тихо не применяются: сначала `cat -A`, либо regex с `[ \t]*`.
5. **`question_result` приходит только когда вопрос закончился** (все ответили или вышло
   время). Чтобы проверить, что игрок получил очки, жди `players_updated`.
6. **Cookie сессии `Secure`**, поэтому `requests` не отправляет её по http — в смоуке она
   прокидывается заголовком `Cookie`, и в WebSocket-хендшейке тоже.
7. **`gh` не авторизован.** Для GitHub API:
   `TOKEN=$(printf 'protocol=https\nhost=github.com\n\n' | git credential fill | sed -n 's/^password=//p')`.
8. **`cargo xtask ci` включает fmt-check** — после ручных правок всегда `cargo fmt --all`.
9. Legacy-приложение на Flask/SQLite живёт в том же репозитории: изменения в общих
   шаблонах/доках не должны ломать `make lint test`.
10. Порт 5000 занят старым процессом после пересборки — перезапускай по `.run/backend.pid`.

## 7. Ближайшие задачи (по приоритету)

**A. Настоящая интеграция OpenRouter (главное).**
Ключ `OPENROUTER_API_KEY` уже в окружении. Нужно:
1. Поднять сервер и убедиться, что провайдер не пропущен: в логах нет
   `skipping provider: no API key configured`, `GET /api/admin/ai/status` показывает
   `openrouter` как активный.
2. Создать комнату и стартовать игру; проверить, что `question_emitted` приходит с
   `source: "ai"`, а не `bank`/`fallback`. Если банк — найти, где роутер откатывается, и
   понять причину (401, модель, таймаут, JSON-схема ответа).
3. Обработать ошибки явно: 401/403 (ключ), 429 (rate limit), таймаут, битый JSON →
   фолбэк на банк + внятный лог **без ключа**. Добавить юнит-тесты на парсинг ответа.
4. Проверить кэш вопросов (`questions_cached` в админке) и что повторная игра с той же
   темой не бьёт по API лишний раз.
5. Не логировать промт целиком на уровне `info`, если в нём могут быть пользовательские
   данные; ключ не должен попадать ни в один лог ни при каких ошибках.

**B. Админка, второй проход.** Выбор модели у провайдера, health-check провайдера
(кнопка «проверить»), экспорт журнала аудита, отображение текущей модели в `/ai/status`.

**C. Игровые режимы и UI.** Довести существующие режимы (`classic`, `ffa`, `lives`,
`blitz`, `team`, `sandbox`), не добавляя новых, пока A не закрыт.

## 8. Definition of done для любой задачи

1. Изменение атомарное и минимальное; лишних файлов и «версий» не создаём.
2. `cargo xtask ci` — зелёный; `svelte-check` — 0/0; затронутое поведение проверено
   смоук-тестом или curl'ом (с выводом в отчёте).
3. Обновлены `CHANGELOG.md` (Unreleased) и релевантный файл в `docs/`.
4. Коммит с `Co-authored-by: openhands <openhands@all-hands.dev>`, пуш в
   `feat/cheat-chat-supabase`, PR #35 обновлён (описание — по факту изменений).
5. В финальном сообщении: что сделано, чем проверено (команды + результат), что осталось.

## 9. Чего не делать

- Не печатать и не коммитить секреты; не создавать `.env` с реальными ключами в git.
- Не пушить в `main`, не мержить PR без явной просьбы.
- Не удалять legacy-приложение и не «чинить» его заодно.
- Не переписывать модуль целиком «одним промптом» — только маленькими проверяемыми шагами.
- Не объявлять задачу выполненной без прогона гейтов.
