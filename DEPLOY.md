# Развёртывание и живучесть

## Почему сайт падает

Песочница агента — **временный контейнер**. Когда сессия закрывается, контейнер
пересоздаётся: вместе с ним исчезают процессы (backend, `vite dev`) и всё, что
установлено в систему (PostgreSQL, Rust). В `/workspace` остаются только файлы.
Поэтому «чтобы не ложился» = вынести сайт на машину, которая живёт независимо.

## Вариант A — VPS + Docker Compose + Caddy (рекомендуется)

Нужно: VPS 2 vCPU / 4 GB (Hetzner CX22, Timeweb, VDSina) и домен.

```bash
# на сервере, один раз
sudo apt-get update && sudo apt-get install -y docker.io docker-compose-plugin git
git clone https://github.com/ForgeLabHub/ForgeBrainStorm.git && cd ForgeBrainStorm

cp deploy/.env.production.example deploy/.env.production
# заполнить DOMAIN, POSTGRES_PASSWORD, SECRET_KEY (openssl rand -hex 32),
# ADMIN_SECRET_KEY, OPENROUTER_API_KEY

docker compose -f deploy/docker-compose.prod.yml --env-file deploy/.env.production up -d --build
```

Что даёт эта конфигурация:

- `restart: unless-stopped` у всех сервисов → упавший контейнер поднимается сам,
  после перезагрузки сервера стек тоже встаёт;
- healthcheck'и (`/health`, `/`, `pg_isready`) → Docker видит зависшие сервисы;
- Caddy сам получает TLS-сертификат и проксирует `/api` и `/ws` (WebSocket) на backend;
- PostgreSQL с volume `pgdata` — данные переживают перезапуски;
- миграции применяются при старте backend.

Обновление версии:

```bash
git pull && docker compose -f deploy/docker-compose.prod.yml --env-file deploy/.env.production up -d --build
```

### Бэкапы и мониторинг

```bash
# ежедневный дамп в /var/backups (crontab -e)
0 4 * * * docker exec brainstorm-postgres-1 pg_dump -U brainstorm brainstorm | gzip > /var/backups/brainstorm-$(date +\%F).sql.gz
```

Мониторинг: UptimeRobot / BetterStack на `https://DOMAIN/health` — письмо, когда сайт недоступен.

## Вариант B — PaaS

В репозитории есть `render.yaml`. Подходит Render/Fly/Railway + managed Postgres.
Минусы: free-tier засыпает, WebSocket-игры требуют платного инстанса.

## Локально / в песочнице

Одна команда поднимает всё с нуля (поставит PostgreSQL и Rust, соберёт, запустит, насыплет демо-данные):

```bash
scripts/bootstrap.sh --seed
```

Сторож, который перезапускает упавшие сервисы, пока контейнер жив:

```bash
scripts/watchdog.sh &          # цикл, проверка каждые 20 с
scripts/watchdog.sh --once     # одиночная проверка (для cron)
```

Cron в песочнице (переживает перезапуск процессов, но не пересоздание контейнера):

```bash
crontab -l 2>/dev/null | { cat; echo "*/2 * * * * cd $PWD && scripts/watchdog.sh --once"; } | crontab -
```

## Демо-аккаунты

`scripts/seed.py` создаёт `uitester`, `wshost`, `anna`, `maxim`, `kirill`
с паролем `supersecret123` и несколько сыгранных партий.
