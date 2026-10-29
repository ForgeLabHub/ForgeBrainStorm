# Соглашения API

## Правила

| Правило | Деталь |
| --- | --- |
| Формат | JSON для `/api/*`, HTML-шаблоны для страниц |
| Живость | `GET /health` — состояние процесса и обеих баз |
| Реальное время | Socket.IO (`app/realtime/socket_events.py`), события комнат/чата |
| Аутентификация | сессия Flask + `SECRET_KEY`; админ-функции требуют `ADMIN_SECRET_KEY` |
| Валидация | `app/validation.py` — единая точка проверки входа |
| Ошибки | `app/errors.py` — единый формат ответа об ошибке |
| CORS | `CORS_ORIGINS`, по умолчанию `*` — **сузьте в проде** |
| Куки | `SESSION_COOKIE_SECURE=true` при HTTPS |

## Группы маршрутов

`/api/rooms`, `/api/leaderboard`, `/api/profile`, `/api/campaign`, `/api/learn`,
`/api/ugc`, `/api/admin`, `/api/cheat` (закрыт кодом `CHEAT_TESTER_CODE`).
Точный список — в `app/http_api/*.py`; README описывает игровые сценарии.

## Безопасность ответов

- Приватные ключи и коды никогда не попадают в ответы (проверяется `tests/test_security.py`).
- Админ-маршруты требуют отдельного секрета, а не только сессии.
- Ошибки не содержат трассировок при `DEBUG=false`.

## Как менять API

1. Изменить маршрут и валидацию. 2. Обновить `static/js/*` (клиент). 3. Прогнать
`make lint test`. 4. Добавить строку в [verification/matrix.md](../verification/matrix.md),
если появилось новое утверждение.
