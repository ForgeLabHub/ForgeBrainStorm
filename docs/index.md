# Документация BrainStorm

Точка входа. Здесь карта документов и правила; существующие документы сохранены.

| Вопрос | Документ |
| --- | --- |
| Запустить с нуля | [../BUILD.md](../BUILD.md) |
| Что уже работает и как играть | [../README.md](../README.md) |
| Правила режимов игры | [game-modes.md](game-modes.md) |
| Как устроен Rust-бэкенд | [rust-backend.md](rust-backend.md) |
| Понять устройство | [architecture/system-context.md](architecture/system-context.md) |
| Где лежат данные | [database/storage.md](database/storage.md) |
| API и ошибки | [api/conventions.md](api/conventions.md) |
| Что делает модуль | [modules/](modules/) |
| Развёртывание | [operations/deployment.md](operations/deployment.md) |
| Наблюдаемость | [operations/monitoring.md](operations/monitoring.md) |
| **Секреты и инцидент с ключом** | [security/secrets.md](security/secrets.md) |
| Чем доказано, что работает | [verification/matrix.md](verification/matrix.md), CI |
| Что исправлено при аудите | [audit/portability-audit-2026-10-29.md](audit/portability-audit-2026-10-29.md) |
| Планы и изменения | [../ROADMAP.md](../ROADMAP.md), [../CHANGELOG.md](../CHANGELOG.md) |

## Что актуально

Продукт — `rust-server/` (axum + tokio + sqlx, WebSocket-игровой цикл) и
`frontend/` (SvelteKit). Их документация: [rust-backend.md](rust-backend.md),
[game-modes.md](game-modes.md), [modules/](modules/), [api/conventions.md](api/conventions.md).

`app/`, `templates/`, `static/`, `run.py`, `wsgi.py` и корневой `Dockerfile` —
**legacy**-монолит Flask + Socket.IO. Он остаётся в репозитории ради старого
деплоя; новые фичи туда не добавляются. Часть документов в `modules/` описывает
именно его — сверяйтесь с кодом `rust-server/`, если сомневаетесь.

## Правила

1. Утверждение без проверки — обещание. См. [verification/matrix.md](verification/matrix.md).
2. Новый модуль без спецификации по [modules/module-template.md](modules/module-template.md)
   не считается готовым (`make docs-check`).
3. Секреты — только через окружение; проверяется `gitleaks` и `make doctor`.
4. Версии зависимостей фиксируются `requirements.lock.txt`; `make lock` — осознанное
   действие, а не автоматика.

## Разработка

- [development/code-style.md](development/code-style.md) — линт, версии инструментов, долг по стилю.

## Управление проектом

- `forge.sh` / `forge.bat` — запуск, остановка, логи, тесты, база и контейнеры одной командой; см. раздел в README.
