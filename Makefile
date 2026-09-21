# ============================================================
# Makefile for solana-zk-pool
# ============================================================

SHELL := /bin/bash
DOCKER_COMPOSE ?= docker compose
COMPOSE_FILE := $(shell pwd)/infra/docker-compose.yml

BACKEND_DIR := /home/ubuntu/services/backend
MERKLE_DIR := /home/ubuntu/services/merkle
PROVER_DIR := /home/ubuntu/services/prover
FRONTEND_DIR := /home/ubuntu/web

SESSION_PREFIX := zkpool

RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
NC := \033[0m

.PHONY: help start stop status attach logs sync-circuits

help:
	@echo "Usage:"
	@echo ""
	@echo "  make start              Start all services (tmux + docker)"
	@echo "  make stop               Stop all services"
	@echo "  make status             Show status"
	@echo "  make attach <name>      Attach to tmux (backend|prover|merkle|frontend)"
	@echo "  make logs <name>        Tail service log"
	@echo "  make sync-circuits      Compile circuits and sync JSON to services"
	@echo ""
	@echo "  Services: backend, merkle, prover, frontend"
	@echo "  Docker:   postgres, redis, solana, prometheus, grafana"

start:
	@$(MAKE) --no-print-directory _start

_start:
	@echo -e "$(BLUE)[INFO]$(NC) Starting Postgres + Redis + solana + Prometheus + Grafana..."
	@$(DOCKER_COMPOSE) -f $(COMPOSE_FILE) up -d postgres redis solana prometheus grafana
	@sleep 3
	@echo -e "$(GREEN)[OK]$(NC) Docker services started"
	@$(MAKE) --no-print-directory _start_merkle
	@$(MAKE) --no-print-directory _start_prover
	@$(MAKE) --no-print-directory _start_backend
	@$(MAKE) --no-print-directory _start_frontend
	@$(MAKE) --no-print-directory _wait_services
	@echo ""
	@echo -e "$(GREEN)[OK]$(NC) All services started!"
	@echo ""
	@echo "  Attach to session:  make attach backend"
	@echo "  Show status:        make status"
	@echo "  Stop all:           make stop"
	@echo ""

_start_merkle:
	@if tmux has-session -t $(SESSION_PREFIX)-merkle 2>/dev/null; then \
		echo -e "$(YELLOW)[WARN]$(NC) Session $(SESSION_PREFIX)-merkle already exists"; \
	else \
		echo -e "$(BLUE)[INFO]$(NC) Starting merkle..."; \
		tmux new-session -d -s $(SESSION_PREFIX)-merkle \
			"$(DOCKER_COMPOSE) -f $(COMPOSE_FILE) exec solana bash -ic 'cd $(MERKLE_DIR) && MERKLE_PORT=4003 node src/server.js' 2>&1 | tee /tmp/zkpool-merkle.log"; \
		echo -e "$(GREEN)[OK]$(NC) merkle → make attach merkle"; \
	fi

_start_prover:
	@if tmux has-session -t $(SESSION_PREFIX)-prover 2>/dev/null; then \
		echo -e "$(YELLOW)[WARN]$(NC) Session $(SESSION_PREFIX)-prover already exists"; \
	else \
		echo -e "$(BLUE)[INFO]$(NC) Starting prover..."; \
		tmux new-session -d -s $(SESSION_PREFIX)-prover \
			"$(DOCKER_COMPOSE) -f $(COMPOSE_FILE) exec solana bash -ic 'cd $(PROVER_DIR) && PROVER_PORT=4002 cargo run' 2>&1 | tee /tmp/zkpool-prover.log"; \
		echo -e "$(GREEN)[OK]$(NC) prover → make attach prover"; \
	fi

_start_backend:
	@if tmux has-session -t $(SESSION_PREFIX)-backend 2>/dev/null; then \
		echo -e "$(YELLOW)[WARN]$(NC) Session $(SESSION_PREFIX)-backend already exists"; \
	else \
		echo -e "$(BLUE)[INFO]$(NC) Starting backend..."; \
		tmux new-session -d -s $(SESSION_PREFIX)-backend \
			"$(DOCKER_COMPOSE) -f $(COMPOSE_FILE) exec solana bash -ic 'cd $(BACKEND_DIR) && cargo run' 2>&1 | tee /tmp/zkpool-backend.log"; \
		echo -e "$(GREEN)[OK]$(NC) backend → make attach backend"; \
	fi

_start_frontend:
	@if tmux has-session -t $(SESSION_PREFIX)-frontend 2>/dev/null; then \
		echo -e "$(YELLOW)[WARN]$(NC) Session $(SESSION_PREFIX)-frontend already exists"; \
	else \
		echo -e "$(BLUE)[INFO]$(NC) Starting frontend..."; \
		tmux new-session -d -s $(SESSION_PREFIX)-frontend \
			"$(DOCKER_COMPOSE) -f $(COMPOSE_FILE) exec solana bash -ic 'cd $(FRONTEND_DIR) && pnpm dev --host 0.0.0.0' 2>&1 | tee /tmp/zkpool-frontend.log"; \
		echo -e "$(GREEN)[OK]$(NC) frontend → make attach frontend"; \
	fi

_wait_services:
	@echo -e "$(BLUE)[INFO]$(NC) Waiting for services to be ready..."
	@for i in $$(seq 1 60); do \
		if curl -s -o /dev/null -w "%{http_code}" http://localhost:4001/api/health 2>/dev/null | grep -q "200"; then \
			echo -e "$(GREEN)[OK]$(NC) backend is ready"; \
			break; \
		fi; \
		sleep 1; \
	done
	@for i in $$(seq 1 30); do \
		if curl -s -o /dev/null -w "%{http_code}" http://localhost:4003/health 2>/dev/null | grep -q "200"; then \
			echo -e "$(GREEN)[OK]$(NC) merkle is ready"; \
			break; \
		fi; \
		sleep 1; \
	done
	@for i in $$(seq 1 30); do \
		if curl -s -o /dev/null -w "%{http_code}" http://localhost:5173 2>/dev/null | grep -q "200"; then \
			echo -e "$(GREEN)[OK]$(NC) frontend is ready"; \
			break; \
		fi; \
		sleep 1; \
	done

stop:
	@echo -e "$(BLUE)[INFO]$(NC) Stopping all services..."
	@for name in prover backend merkle frontend; do \
		session="$(SESSION_PREFIX)-$$name"; \
		if tmux has-session -t "$$session" 2>/dev/null; then \
			tmux kill-session -t "$$session"; \
			echo -e "$(GREEN)[OK]$(NC) Stopped session $$session"; \
		fi; \
	done
	@$(DOCKER_COMPOSE) -f $(COMPOSE_FILE) stop postgres redis
	@echo -e "$(GREEN)[OK]$(NC) All stopped"

status:
	@echo ""
	@echo "=== tmux sessions ==="
	@for name in prover backend merkle frontend; do \
		session="$(SESSION_PREFIX)-$$name"; \
		if tmux has-session -t "$$session" 2>/dev/null; then \
			echo -e "  $(GREEN)✓$(NC) $$session"; \
		else \
			echo -e "  $(RED)✗$(NC) $$session"; \
		fi; \
	done
	@echo ""
	@echo "=== HTTP health ==="
	@for url in \
		"http://localhost:4001/api/health|backend" \
		"http://localhost:4003/health|merkle" \
		"http://localhost:5173|frontend" \
		"http://localhost:9090/-/healthy|prometheus" \
		"http://localhost:3000/api/health|grafana"; do \
		endpoint="$${url%|*}"; \
		name="$${url#*|}"; \
		code=$$(curl -s -o /dev/null -w "%{http_code}" "$$endpoint" 2>/dev/null || echo "000"); \
		if [ "$$code" = "200" ]; then \
			echo -e "  $(GREEN)✓$(NC) $$name ($$code)"; \
		else \
			echo -e "  $(RED)✗$(NC) $$name ($$code)"; \
		fi; \
	done
	@echo ""
	@echo "=== Docker ==="
	@$(DOCKER_COMPOSE) -f $(COMPOSE_FILE) ps 2>/dev/null || true
	@echo ""

attach:
	@if [ -z "$(filter-out $@,$(MAKECMDGOALS))" ]; then \
		echo "Specify session name: backend | prover | merkle | frontend"; \
		exit 1; \
	fi
	@session="$(SESSION_PREFIX)-$(filter-out $@,$(MAKECMDGOALS))"; \
	if ! tmux has-session -t "$$session" 2>/dev/null; then \
		echo "Session $$session not found"; \
		exit 1; \
	fi; \
	tmux attach -t "$$session"

logs:
	@if [ -z "$(filter-out $@,$(MAKECMDGOALS))" ]; then \
		echo "Specify service name: backend | prover | merkle | frontend"; \
		exit 1; \
	fi
	@logfile="/tmp/zkpool-$(filter-out $@,$(MAKECMDGOALS)).log"; \
	if [ ! -f "$$logfile" ]; then \
		echo "Log $$logfile not found"; \
		exit 1; \
	fi; \
	tail -f "$$logfile"

sync-circuits:
	@echo -e "$(BLUE)[INFO]$(NC) Syncing circuits..."
	@$(DOCKER_COMPOSE) -f $(COMPOSE_FILE) exec solana bash -ic 'cd /home/ubuntu && cargo run --manifest-path scripts/sync-circuits/Cargo.toml'
	@echo -e "$(GREEN)[OK]$(NC) Circuits synced"

%:
	@:
