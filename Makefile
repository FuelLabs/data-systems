# ------------------------------------------------------------
#  Setup
# ------------------------------------------------------------

TARGET ?= aarch64-apple-darwin
PACKAGE ?= fuel-core-nats

.PHONY: all build run clean lint fmt help test doc

all: build

install:
	cargo fetch

setup:
	./scripts/setup.sh

# ------------------------------------------------------------
#  Development
# ------------------------------------------------------------

dev:
	cargo run

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
