# Makefile for identity service development tasks
.PHONY: help install-sqlx migrate run docker-build docker-up test

help:
	@echo "Available targets:"
	@echo "  install-sqlx   Install sqlx-cli (requires cargo)"
	@echo "  migrate        Run migrations using sqlx migrate if available, else psql"
	@echo "  run            Run service locally (requires DATABASE_URL and JWT_SECRET)"
	@echo "  docker-build   Build Docker image for identity service"
	@echo "  docker-up      Start dev environment with docker-compose.dev.yml"
	@echo "  test           Run unit and integration tests"

install-sqlx:
	@if command -v sqlx >/dev/null 2>&1; then \
		echo "sqlx already installed"; \
	else \
		cargo install sqlx-cli --no-default-features --features postgres || true; \
	fi

migrate:
	@if command -v sqlx >/dev/null 2>&1; then \
		echo "Running migrations with sqlx migrate run"; \
		sqlx migrate run; \
	else \
		if [ -z "${DATABASE_URL}" ]; then echo "DATABASE_URL required for psql fallback"; exit 1; fi; \
		echo "sqlx not found; applying .sql files using psql"; \
		for f in ./sql/migrations/*.sql; do echo "Applying $$f"; psql "$${DATABASE_URL}" -f $$f; done; \
	fi

run:
	@echo "Run identity service (dev) - ensure DATABASE_URL and JWT_SECRET are set"
	cargo run

docker-build:
	docker build -t identity_service:local .

docker-up:
	docker-compose -f docker-compose.dev.yml up --build

test:
	cargo test
