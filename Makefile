# ------------------------------------------------------------
#  Setup
# ------------------------------------------------------------

PACKAGE ?= fuel-core-nats
TARGET ?= aarch64-apple-darwin
DOCKER_PROFILE ?= all

.PHONY: all build run clean lint fmt help setup start-nats stop-nats test doc

all: build

install:
	cargo fetch

check-commands:
	@for cmd in $(COMMANDS); do \
		if ! command -v $$cmd >/dev/null 2>&1; then \
			echo >&2 "$$cmd is not installed. Please install $$cmd and try again.."; \
			exit 1; \
		fi \
	done

setup: COMMANDS=rustup pre-commit
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
start/fuel-core stop/fuel-core restart/fuel-core clean/fuel-core: DOCKER_PROFILE = fuel

start/nats start/fuel-core: start
stop/nats stop/fuel-core: stop
restart/nats restart/fuel-core: restart
clean/nats clean/fuel-core: clean/docker

dev-watch:
	cargo watch -- cargo run

# ------------------------------------------------------------
# Build & Release
# ------------------------------------------------------------

build: install
	cargo build --release --target "$(TARGET)" --package "$(PACKAGE)"

run:
	cargo run --release

clean:
	cargo clean
	rm -rf release

# ------------------------------------------------------------
# Format
# ------------------------------------------------------------

fmt: fmt-cargo fmt-rust fmt-markdown

fmt-cargo:
	cargo sort -w

fmt-rust:
	cargo +nightly fmt -- --color always

fmt-markdown:
	npx prettier *.md **/*.md --write --no-error-on-unmatched-pattern

# ------------------------------------------------------------
# Validate code
# ------------------------------------------------------------

check:
	cargo check --all-targets --all-features

lint: check lint-cargo lint-rust lint-clippy lint-markdown

lint-cargo:
	cargo sort -w --check

lint-rust:
	cargo +nightly fmt -- --check --color always

lint-clippy:
	cargo clippy --workspace -- -D warnings

lint-markdown:
	npx prettier *.md **/*.md --check --no-error-on-unmatched-pattern

# ------------------------------------------------------------
# Test
# ------------------------------------------------------------

test:
	cargo test --all

# ------------------------------------------------------------
# Documentation
# ------------------------------------------------------------

doc:
	cargo doc --no-deps

# ------------------------------------------------------------
# Help
# ------------------------------------------------------------

help:
	@echo "Available commands:"
	@echo "  install   - Install project dependencies"
	@echo "  setup     - Run the setup script"
	@echo "  dev       - Run the project in development mode"
	@echo "  dev-watch - Run the project in development mode with auto-reload"
	@echo "  build     - Build the project"
	@echo "  run       - Run the project in release mode"
	@echo "  clean     - Clean the build artifacts and release directory"
	@echo "  fmt       - Format the code and Markdown files"
	@echo "  lint      - Perform linting checks on the code and Markdown files"
	@echo "  test      - Run tests"
	@echo "  doc       - Generate documentation"
