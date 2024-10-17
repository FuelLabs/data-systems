# ------------------------------------------------------------
#  Setup
# ------------------------------------------------------------

PACKAGE ?= fuel-streams-publisher
DOCKER_PROFILE ?= all
RUST_NIGHTLY_VERSION ?= nightly-2024-07-28
DOCKER_COMPOSE = docker compose -f docker/docker-compose.yml
PROFILES = dev nats publisher monitoring indexer

# Phony targets
.PHONY: all install setup build clean lint fmt help test doc bench coverage audit \
        $(foreach p,$(PROFILES),start/$(p) stop/$(p) restart/$(p) clean/$(p)) \
        start stop restart clean/docker

# Default target
all: build

install:
	cargo fetch

check-commands:
	@for cmd in $(COMMANDS); do \
		if ! command -v $$cmd >/dev/null 2>&1; then \
			echo >&2 "$$cmd is not installed. Please install $$cmd and try again."; \
			exit 1; \
		fi \
	done

setup: COMMANDS=rustup npm pre-commit
setup: check-commands
	./scripts/setup.sh

# ------------------------------------------------------------
#  Development
# ------------------------------------------------------------

start: check-commands
	$(DOCKER_COMPOSE) --profile $(DOCKER_PROFILE) up -d

stop: check-commands
	$(DOCKER_COMPOSE) --profile $(DOCKER_PROFILE) down

restart: stop start

clean/docker: stop
	$(DOCKER_COMPOSE) --profile $(DOCKER_PROFILE) down -v --rmi all --remove-orphans

define profile_rules
start/$(1) stop/$(1) restart/$(1) clean/$(1): DOCKER_PROFILE = $(1)
start/$(1): start
stop/$(1): stop
restart/$(1): restart
clean/$(1): clean/docker
endef

$(foreach p,$(PROFILES),$(eval $(call profile_rules,$(p))))

dev-watch:
	cargo watch -- cargo run

# ------------------------------------------------------------
# Formatting
# ------------------------------------------------------------

fmt: fmt-cargo fmt-rust fmt-prettier fmt-markdown

fmt-cargo:
	cargo sort -w

fmt-rust:
	cargo +$(RUST_NIGHTLY_VERSION) fmt -- --color always

fmt-prettier:
	pnpm prettier:fix

fmt-markdown:
	pnpm md:fix

# ------------------------------------------------------------
# Validate code
# ------------------------------------------------------------

check:
	cargo check --all-targets --all-features

lint: check lint-cargo lint-rust lint-clippy lint-prettier lint-markdown

lint-cargo:
	cargo sort -w --check

lint-rust:
	cargo +$(RUST_NIGHTLY_VERSION) fmt -- --check --color always

lint-clippy:
	cargo clippy --workspace -- -D warnings

lint-prettier:
	pnpm prettier:validate

lint-markdown:
	pnpm md:lint

# ------------------------------------------------------------
# Coverage
# ------------------------------------------------------------

coverage:
	RUSTFLAGS="-Z threads=8" cargo +$(RUST_NIGHTLY_VERSION) tarpaulin --config ./tarpaulin.toml

# ------------------------------------------------------------
# Audit crates
# ------------------------------------------------------------

audit:
	cargo audit

audit-fix-test:
	cargo audit fix --dry-run

audit-fix:
	cargo audit fix

# ------------------------------------------------------------
# Build, Test, and Documentation
# ------------------------------------------------------------

build:
	cargo build --package $(PACKAGE)

test:
	cargo nextest run --workspace --color always --locked

doc:
	cargo doc --no-deps

# ------------------------------------------------------------
# Benchmarking
# ------------------------------------------------------------

bench:
	cargo bench -p data-parser -p nats-publisher -p bench-consumers

# ------------------------------------------------------------
# Help
# ------------------------------------------------------------

help:
	@echo "Available commands:"
	@echo "  install           - Install project dependencies"
	@echo "  setup             - Run the setup script"
	@echo "  start             - Start Docker containers"
	@echo "  stop              - Stop Docker containers"
	@echo "  restart           - Restart Docker containers"
	@echo "  dev-watch         - Run the project in development mode with auto-reload"
	@echo "  build             - Build the project"
	@echo "  clean             - Clean Docker containers and images"
	@echo "  fmt               - Format the code, Markdown, and YAML files"
	@echo "  lint              - Perform linting checks on the code and Markdown files"
	@echo "  audit             - Perform audit checks on Rust crates"
	@echo "  test              - Run tests"
	@echo "  doc               - Generate documentation"
	@echo "  bench             - Run benchmarks"
	@echo ""
	@echo "Docker service commands:"
	@echo "  start/nats        - Start NATS Docker container"
	@echo "  stop/nats         - Stop NATS Docker container"
	@echo "  restart/nats      - Restart NATS Docker container"
	@echo "  clean/nats        - Remove NATS Docker container and images"
	@echo "  start/publisher   - Start Publisher Docker container"
	@echo "  stop/publisher    - Stop Publisher Docker container"
	@echo "  restart/publisher - Restart Publisher Docker container"
	@echo "  clean/publisher   - Remove Publisher Docker container and images"
	@echo "  start/monitoring  - Start Monitoring Docker containers"
	@echo "  stop/monitoring   - Stop Monitoring Docker containers"
	@echo "  restart/monitoring - Restart Monitoring Docker containers"
	@echo "  clean/monitoring  - Remove Monitoring Docker containers and images"
	@echo "  start/surrealdb   - Start SurrealDB Docker container"
	@echo "  stop/surrealdb    - Stop SurrealDB Docker container"
	@echo "  restart/surrealdb - Restart SurrealDB Docker container"
	@echo "  clean/surrealdb   - Remove SurrealDB Docker container and images"
