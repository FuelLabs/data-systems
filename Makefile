# ------------------------------------------------------------
#  Setup
# ------------------------------------------------------------

PACKAGE ?= fuel-core-nats
ifeq ($(CI),true)
    TARGET ?= x86_64-unknown-linux-gnu
else
    TARGET ?= aarch64-apple-darwin
endif

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

start:	COMMANDS=docker
start:	check-commands
	docker compose -f docker/docker-compose.yml up

stop:
	docker compose -f docker/docker-compose.yml down

start/nats:	COMMANDS=docker
start/nats:	COMMANDS=docker
start/nats: check-commands
	docker run -p 4222:4222 -p 8222:8222 -p 6222:6222 \
	--mount type=bind,source="$$(pwd)"/crates/fuel-core-nats/nats.conf,target=/etc/nats/nats.conf \
	--env-file .env \
	--name fuel-core-nats-server \
	$(if $(CI),,--tty --interactive) \
	--detach \
	nats:latest --js --config /etc/nats/nats.conf

stop/nats:
	docker rm -f $$(docker ps -a -q --filter ancestor=nats:latest)

# Starts fuel-core-nats service
start/fuel-core:
	./scripts/start-fuel-core-nats.sh

restart/nats: stop/nats start/nats
restart/fuel-core: stop/fuel-core start/fuel-core

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
