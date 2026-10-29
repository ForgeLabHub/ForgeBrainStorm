# Как участвовать в BrainStorm

Русская версия [../CONTRIBUTING.md](../CONTRIBUTING.md) (английский оригинал).

- [Кодекс поведения](../CODE_OF_CONDUCT.md)
- [Политика безопасности](../SECURITY.md) — об уязвимостях сообщайте **приватно**, а не
  публичным issue

## Настройка разработки

```bash
git clone https://github.com/geff1778/Brainstorm-test.git
cd Brainstorm-test

python -m venv .venv
source .venv/bin/activate        # Windows: .venv\Scripts\activate

pip install -r requirements.txt
pip install -r requirements-dev.txt
cp .env.example .env             # заполните значения; .env не коммитится
```

Запуск и проверки:

```bash
python run.py                    # http://localhost:5000
pytest                           # полный набор (190 тестов)
pytest --cov=app --cov-report=term-missing
ruff check app tests             # линт
ruff format app tests            # форматирование
```

Тесты идут на стандартном режиме Socket.IO `threading` и на временном `DATA_DIR`, поэтому
внешние сервисы не нужны.

## Ветки и коммиты

- Ветвитесь от `main`. Имена: `feat/…`, `fix/…`, `docs/…`, `refactor/…`.
- Один коммит — одно изменение. Заголовок в императиве: `fix: guard double question resolve`.
- Никогда не коммитьте `.env`, `data/`, базы и любые учётные данные — `.gitignore`
  покрывает типовые случаи.
- Не пушьте в `main` напрямую, открывайте pull request.

## Правила архитектуры

Направление зависимостей:

```
config / security / validation / errors / logging_setup   (сквозные)
        ▲
domain  (чистая игровая логика — без I/O, сети и БД)
        ▲
services (логика приложения; может использовать domain + db)
        ▲
realtime / http_api (тонкие адаптеры над services)
```

- `domain` остаётся чистым и не импортирует `services`, `realtime`, `http_api`.
- Побочные эффекты (БД, сеть, Socket.IO) — в `services`, `realtime`, `http_api`.
- Маленькие функции и внедрение зависимостей предпочтительнее глобальных переменных.
