# Сборка: от чистой машины до работающей игры

## Закреплённые версии

| Инструмент | Версия | Источник истины |
| --- | --- | --- |
| Python | 3.12 | `Dockerfile`, `ci.yml`, `scripts/doctor.sh` |
| Python-зависимости | `requirements.lock.txt` (37 пакетов, точные версии) | `make install` |
| Диапазоны для обновления | `requirements.txt` | `make lock` |
| Dev-инструменты | `pytest`, `pytest-cov`, `ruff` | `ci.yml` |
| Runtime-сервер | gunicorn + eventlet (`gunicorn.conf.py`) | прод-запуск |

## Уровень A — исходный код

Ни Docker, ни GigaChat, ни внешняя БД не нужны: данные в SQLite, AI выключен.

```bash
make doctor       # инструменты, версии, отсутствие .env в git, зависимости
make install-dev  # закреплённые зависимости + pytest/ruff
make lint         # ruff
make test         # 190 тестов
make docs-check
```

## Уровень B — артефакты

Python-приложение не компилируется: артефакт — образ.

```bash
make build-image  # docker build -t brainstorm:local .
```

## Уровень C — интеграционная среда

```bash
cp .env.example .env    # минимум: SECRET_KEY; AI не обязателен
make compose-up
make smoke-test
```

## Уровень D — контейнерная проверка

CI поднимает контейнер и опрашивает `/health` (`ci.yml`, job `docker`). Локально:

```bash
make build-image
docker run -d -p 5000:5000 -e SECRET_KEY=dev-secret-key-that-is-long-enough \
  -e ADMIN_SECRET_KEY=1777 -e CHEAT_TESTER_CODE=19112009 brainstorm:local
./scripts/smoke-test.sh http://localhost:5000
```

## Уровень E — чистое развёртывание

```bash
docker compose down -v          # осознанно: удаляет том с данными
make compose-up
make smoke-test
# копия тома и восстановление — см. docs/database/storage.md
```

## Переменные окружения

Обязательные для compose: `SECRET_KEY`, `ADMIN_SECRET_KEY`, `CHEAT_TESTER_CODE`.
Опциональные: `GIGACHAT_*` (без них AI выключен), `CORS_ORIGINS`,
`SESSION_COOKIE_SECURE`, `SOCKETIO_ASYNC_MODE`, `LOG_LEVEL`, `DATA_DIR`.

## Типовые ошибки

| Ошибка | Причина | Решение |
| --- | --- | --- |
| `set SECRET_KEY in your environment` | нет `.env` | `cp .env.example .env` |
| AI-функции не работают | `GIGACHAT_CREDENTIALS` пуст | это ожидаемый режим; заполнить при необходимости |
| `database is locked` | конкурирующие записи в SQLite | используется WAL и короткие соединения; проверить нагрузку |
| Тесты пишут в реальные данные | не задан `DATA_DIR` | `tests/conftest.py` ставит временный каталог; не переопределяйте |
| Секрет попал в коммит | `.env` в индексе | `git rm --cached .env`, ротация ключа, см. `docs/security/secrets.md` |

## Доказательство

`ci.yml` покрывает уровни A–D. Уровень E (копия/восстановление) описан и выполняется
вручную — это зафиксировано в матрице как непроверяемое автоматически.
