# ------------------------------------------------------------
#  Setup
# ------------------------------------------------------------

PACKAGE ?= fuel-streams-publisher
DOCKER_PROFILE ?= all
RUST_NIGHTLY_VERSION ?= nightly-2024-07-28

.PHONY: all build clean lint fmt help setup start stop restart clean/docker start/nats stop/nats restart/nats clean/nats start/fuel-core stop/fuel-core restart/fuel-core clean/fuel-core dev-watch fmt-cargo fmt-rust fmt-markdown fmt-yaml check lint-cargo lint-rust lint-clippy lint-markdown audit audit-fix-test audit-fix test doc bench

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
	docker compose --profile $(DOCKER_PROFILE) -f docker/docker-compose.yml up -d

stop: check-commands
	docker compose --profile $(DOCKER_PROFILE) -f docker/docker-compose.yml down

restart: stop start

clean/docker: stop
	docker compose --profile $(DOCKER_PROFILE) -f docker/docker-compose.yml down -v --rmi all --remove-orphans

start/nats stop/nats restart/nats clean/nats: DOCKER_PROFILE = nats
start/publisher stop/publisher restart/publisher clean/publisher: DOCKER_PROFILE = fuel

start/nats start/publisher: start
stop/nats stop/publisher: stop
restart/nats restart/publisher: restart
clean/nats clean/publisher: clean/docker

dev-watch:
	cargo watch -- cargo run

# ------------------------------------------------------------
# Formatting
# ------------------------------------------------------------

fmt: fmt-cargo fmt-rust fmt-prettier

fmt-cargo:
	cargo sort -w

fmt-rust:
	cargo +$(RUST_NIGHTLY_VERSION) fmt -- --color always

fmt-prettier:
	pnpm prettier:fix

# ------------------------------------------------------------
# Validate code
# ------------------------------------------------------------

check:
	cargo check --all-targets --all-features

lint: check lint-cargo lint-rust lint-clippy lint-prettier

lint-cargo:
	cargo sort -w --check

lint-rust:
	cargo +$(RUST_NIGHTLY_VERSION) fmt -- --check --color always

lint-clippy:
	cargo clippy --workspace -- -D warnings

lint-prettier:
	pnpm prettier:validate

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
	cargo test --workspace

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
	@echo "  install     - Install project dependencies"
	@echo "  setup       - Run the setup script"
	@echo "  start       - Start Docker containers"
	@echo "  stop        - Stop Docker containers"
	@echo "  restart     - Restart Docker containers"
	@echo "  dev-watch   - Run the project in development mode with auto-reload"
	@echo "  build       - Build the project"
	@echo "  clean       - Clean Docker containers and images"
	@echo "  fmt         - Format the code, Markdown, and YAML files"
	@echo "  lint        - Perform linting checks on the code and Markdown files"
	@echo "  audit       - Perform audit checks on Rust crates"
	@echo "  test        - Run tests"
	@echo "  doc         - Generate documentation"
	@echo "  bench       - Run benchmarks"
