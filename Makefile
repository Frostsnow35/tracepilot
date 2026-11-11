.PHONY: help db backend frontend test lint format check docker-up docker-down deploy deploy-down seed

help:
	@echo "TracePilot dev commands"
	@echo "  make db          - start Postgres + TimescaleDB"
	@echo "  make backend     - run Rust backend (cargo run)"
	@echo "  make frontend    - run Next.js dev server"
	@echo "  make test        - cargo test (backend)"
	@echo "  make lint        - cargo clippy -D warnings"
	@echo "  make format      - cargo fmt"
	@echo "  make check       - fmt --check + clippy + test"
	@echo "  make docker-up   - start full stack"
	@echo "  make docker-down - stop full stack"
	@echo "  make deploy      - one-shot deploy: build + up + healthcheck"
	@echo "  make deploy-down - stop and remove containers (keep data)"
	@echo "  make seed        - generate demo traces into running instance"

db:
	docker compose up -d db

backend:
	cd backend && cargo run

frontend:
	cd frontend && npm run dev

test:
	cd backend && cargo test

lint:
	cd backend && cargo clippy --all-targets -- -D warnings

format:
	cd backend && cargo fmt

check:
	cd backend && cargo fmt --all -- --check
	cd backend && cargo clippy --all-targets -- -D warnings
	cd backend && cargo test

docker-up:
	docker compose up -d

docker-down:
	docker compose down

deploy:
	bash scripts/deploy.sh

deploy-down:
	bash scripts/deploy.sh --down

seed:
	python scripts/seed_demo.py
