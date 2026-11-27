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
        compose-up compose-down compose-logs smoke-test docs-check docs-pdf lock verify clean \
        rust-fmt rust-check rust-lint rust-test rust-verify frontend-check agent-verify agent-verify-all \
        ws-check ws-check-full

help: ## Список целей
	@grep -hE '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) \
	  | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

doctor: ## Проверить инструменты, версии и конфигурацию (ничего не меняет)
	@./scripts/doctor.sh

install: ## Установить закреплённые зависимости
	$(PYTHON) -m pip install -r requirements.lock.txt

install-dev: ## Установить зависимости для разработки (pytest, ruff)
	$(PYTHON) -m pip install -r requirements-dev.txt

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

ws-check: ## Прогнать WebSocket-протокол игры на живом инстансе (нужен websocket-client)
	@$(PYTHON) scripts/ws-check.py --base $(BASE_URL)

ws-check-full: ## То же + полная партия, повторная игра и режим на выживание (~3 мин)
	@$(PYTHON) scripts/ws-check.py --base $(BASE_URL) --rematch --lives

docs-pdf: ## Собрать PDF документации (оба языка)
	$(PYTHON) tools/docs_pdf.py

docs-check: ## Проверить документацию, переводы и PDF
	@./scripts/docs-check.sh
	$(PYTHON) tools/docs_pdf.py --check

verify: lint test docs-check ## Локальная матрица
	@echo "verify: ok (образ и compose проверяются в CI)"

# --- Rust server (rust-server/) ---------------------------------------------
RUST_DIR ?= rust-server
CARGO    ?= cargo

rust-fmt: ## rustfmt --check
	cd $(RUST_DIR) && $(CARGO) fmt --all -- --check

rust-check: ## cargo check --all-targets
	cd $(RUST_DIR) && $(CARGO) check --all-targets --locked

rust-lint: ## clippy, warnings are errors
	cd $(RUST_DIR) && $(CARGO) clippy --all-targets --locked -- -D warnings

rust-test: ## cargo test
	cd $(RUST_DIR) && $(CARGO) test --locked

rust-verify: rust-fmt rust-check rust-lint rust-test ## Rust matrix

frontend-check: ## svelte-check + production build
	cd frontend && npm run check && npm run build

agent-verify: rust-verify ## Gate for AI agents: run before reporting success
	@echo "agent-verify: rust ok (fmt, check, clippy -D warnings, tests)"

agent-verify-all: agent-verify lint test frontend-check ## Everything an agent can break
	@echo "agent-verify-all: python, rust and frontend are green"

clean: ## Удалить артефакты
	rm -rf .pytest_cache .coverage htmlcov
	find . -name __pycache__ -type d -prune -exec rm -rf {} +
