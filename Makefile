# BrainStorm — единая точка входа для сборки и проверок.
#
# Цели уровня A (lint/test/run) работают без Docker и без внешних сервисов:
# приложение использует SQLite в DATA_DIR и не требует GigaChat.

SHELL := /usr/bin/env bash
.DEFAULT_GOAL := help

PYTHON       ?= python3
PYTHON_VER   := 3.12
COMPOSE      ?= docker compose
PORT         ?= 5000
BASE_URL     ?= http://localhost:$(PORT)

.PHONY: help doctor install install-dev run test lint coverage build-image \
        compose-up compose-down compose-logs smoke-test docs-check lock verify clean

help: ## Список целей
	@grep -hE '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) \
	  | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

doctor: ## Проверить инструменты, версии и конфигурацию (ничего не меняет)
	@./scripts/doctor.sh

install: ## Установить закреплённые зависимости
	$(PYTHON) -m pip install -r requirements.lock.txt

install-dev: ## Установить зависимости для разработки (pytest, ruff)
	$(PYTHON) -m pip install -r requirements.lock.txt pytest pytest-cov ruff

lock: ## Пересобрать requirements.lock.txt из requirements.txt (осознанно!)
	@$(PYTHON) -m pip install --dry-run --ignore-installed --quiet \
	  --report /tmp/brainstorm-report.json -r requirements.txt
	@$(PYTHON) tools/make_lock.py /tmp/brainstorm-report.json > requirements.lock.txt
	@echo "requirements.lock.txt обновлён — прогоните make test перед коммитом"

run: ## Запустить dev-сервер (python run.py)
	$(PYTHON) run.py

test: ## Все тесты
	$(PYTHON) -m pytest -q

coverage: ## Тесты с покрытием
	$(PYTHON) -m pytest -q --cov=app --cov-report=term-missing

lint: ## ruff
	$(PYTHON) -m ruff check app tests

build-image: ## Собрать production-образ
	docker build -t brainstorm:local .

compose-up: ## Поднять контейнер
	$(COMPOSE) up -d --build

compose-down: ## Остановить (данные в томе сохраняются)
	$(COMPOSE) down

compose-logs: ## Логи
	$(COMPOSE) logs -f --tail=200

smoke-test: ## Проверить живой инстанс
	@./scripts/smoke-test.sh $(BASE_URL)

docs-check: ## Проверить документацию и спецификации модулей
	@./scripts/docs-check.sh

verify: lint test docs-check ## Локальная матрица
	@echo "verify: ok (образ и compose проверяются в CI)"

clean: ## Удалить артефакты
	rm -rf .pytest_cache .coverage htmlcov
	find . -name __pycache__ -type d -prune -exec rm -rf {} +
