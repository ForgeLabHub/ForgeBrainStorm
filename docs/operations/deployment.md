# Развёртывание

## Способы

| Способ | Файл | Комментарий |
| --- | --- | --- |
| Docker Compose (рекомендуется) | `docker-compose.yml` | один сервис, том `brainstorm-data` |
| Контейнер вручную | `Dockerfile` | multi-stage, non-root `appuser` |
| Process manager | `Procfile`, `gunicorn.conf.py` | `gunicorn -c gunicorn.conf.py wsgi:app` |
| Ручной запуск | `run.py` | только для разработки |

## Порядок

```bash
cp .env.example .env
# заполнить: SECRET_KEY (обязательно), ADMIN_SECRET_KEY, CHEAT_TESTER_CODE
make doctor
make compose-up
make smoke-test BASE_URL=http://localhost:5000
```

Compose падает, если обязательные секреты не заданы (`${VAR:?…}`) — это защита от
развёртывания с пустыми ключами.

## Перед продом

1. **Ротировать секреты**, если репозиторий когда-либо был публичным: см.
   [../security/secrets.md](../security/secrets.md).
2. `SESSION_COOKIE_SECURE=true` и HTTPS на прокси.
3. `CORS_ORIGINS` — точный список доменов вместо `*`.
4. `DEBUG=false` и `ENV=production` (иначе трассировки и подробные ошибки).
5. `SOCKETIO_ASYNC_MODE=eventlet` для нагрузки; `threading` — запасной вариант без
   дополнительных зависимостей.
6. Один процесс: комнаты и планировщик живут в памяти. Несколько реплик без общего
   состояния не поддерживаются.
7. Резервные копии тома `brainstorm-data`, см.
   [../database/storage.md](../database/storage.md).

## Проверка после развёртывания

```bash
make smoke-test BASE_URL=https://<домен>
curl -s https://<домен>/health | python -m json.tool
```
