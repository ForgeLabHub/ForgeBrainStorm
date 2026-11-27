# Supabase

BrainStorm работает на обычном Postgres, поэтому Supabase — это тот же
`DATABASE_URL`, только хост в облаке. Никакого отдельного кода для «режима
Supabase» нет: важны две вещи — TLS и режим пулера.

## 1. Строка подключения

Dashboard → Project Settings → Database → Connection string.

| Что | Порт | Когда использовать |
| --- | --- | --- |
| Session pooler (`pooler.supabase.com`) | 5432 | миграции, долгие соединения |
| Transaction pooler (`pooler.supabase.com`) | 6543 | приложение в проде |
| Direct (`db.<ref>.supabase.co`) | 5432 | IPv6-only, не для serverless |

```
DATABASE_URL=postgres://postgres.<ref>:<password>@aws-0-<region>.pooler.supabase.com:6543/postgres
```

Сервер сам делает две вещи (`crates/server/src/db.rs`):

* **добавляет `sslmode=require`**, если хост облачный, а параметра нет —
  Supabase отказывает в открытом соединении, и это самая частая ошибка
  первого запуска;
* **отключает кэш prepared statements** (`statement_cache_capacity(0)`), если
  в URL есть `pooler.supabase.com` или порт `6543`: транзакционный пулер
  отдаёт соединение другому клиенту между запросами, и именованные
  подготовленные запросы перестают существовать.

Локальный `postgres://localhost/...` остаётся без TLS — для docker compose
ничего не меняется.

## 2. Миграции

Миграции те же, что и для локальной базы, и применяются на старте:

```bash
DATABASE_URL="postgres://...pooler.supabase.com:5432/postgres" cargo run -p brainstorm
```

Для прод-пулера (`:6543`) миграции тоже проходят — они не используют
подготовленные запросы с именами. Если провайдер всё же ругается, прогоните
миграции через session pooler, а приложение переключите на `:6543`.

Расширение `citext` создаётся первой миграцией; на Supabase оно доступно
(схема `extensions`), отдельная установка не нужна.

## 3. MCP

`.mcp.json` в корне подключает официальный сервер
`@supabase/mcp-server-supabase` в режиме только для чтения:

```bash
export SUPABASE_ACCESS_TOKEN=...   # https://supabase.com/dashboard/account/tokens
export SUPABASE_PROJECT_REF=...    # ref из URL проекта
```

Токен и ref держите в окружении, не в репозитории: `.env` в `.gitignore`,
`.mcp.json` содержит только подстановки `${...}`.

Через MCP удобно смотреть схему, выполнять `select` и проверять, что миграции
доехали. Запись (`--read-only` убран) включайте только осознанно: агент
сможет менять прод-данные.

## 4. Проверка

```bash
curl -s localhost:5000/readyz      # {"db":"ok"}
```

`/readyz` делает `SELECT 1` через пул, то есть проверяет и TLS, и пулер.
