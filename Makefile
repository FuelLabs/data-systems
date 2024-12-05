# ------------------------------------------------------------
#  Core Variables (Simple Assignment)
# ------------------------------------------------------------

PACKAGE := fuel-streams
VERSION := $(shell cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "$(PACKAGE)") | .version')
TILTFILE := ./Tiltfile

# ------------------------------------------------------------
#  Tool Versions (Simple Assignment)
# ------------------------------------------------------------

RUST_VERSION := 1.81.0
RUST_NIGHTLY_VERSION := nightly-2024-11-06

# ------------------------------------------------------------
#  Required Commands and Tools (Simple Assignment)
# ------------------------------------------------------------

COMMANDS := rustup npm pre-commit docker python3

# ------------------------------------------------------------
#  Docker Configuration (Simple Assignment)
# ------------------------------------------------------------

NETWORKS := mainnet testnet
PROFILES := all dev nats fuel monitoring indexer logging
MODES := dev profiling
DOCKER_COMPOSE := ./scripts/set_envs.sh && docker compose -f docker/docker-compose.yml --env-file .env

# ------------------------------------------------------------
#  Phony Targets Declaration
# ------------------------------------------------------------

.PHONY: all install setup build clean lint fmt help test doc bench coverage audit \
        version bump-version release release-dry-run docs docs-serve \
        test-watch validate-env dev-watch ci \
        fmt-cargo fmt-rust fmt-prettier fmt-markdown \
        check lint-cargo lint-rust lint-clippy lint-prettier lint-markdown lint-machete \
        coverage audit audit-fix audit-fix-test \
        clean/build clean/docker \
        check-network check-versions check-dev-env check-commands \
        $(foreach p,$(PROFILES),start/$(p) stop/$(p) restart/$(p) clean/$(p)) \
        $(foreach n,$(NETWORKS),start-$(n) stop-$(n) restart-$(n) clean-$(n)) \
        $(foreach n,$(NETWORKS),$(foreach p,$(PROFILES),start-$(n)/$(p) stop-$(n)/$(p) restart-$(n)/$(p) clean-$(n)/$(p))) \
        $(foreach n,$(NETWORKS),$(foreach m,$(MODES),run-$(n)-$(m))) \
        start stop restart run-publisher

# Default target
all: help

# ------------------------------------------------------------
#  Version Management
# ------------------------------------------------------------

version:
	@echo "Current version: $(VERSION)"

bump-version: NEW_VERSION ?=
bump-version:
	@if [ -z "$(NEW_VERSION)" ]; then \
		echo "Error: NEW_VERSION is required"; \
		echo "Usage: make bump-version NEW_VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "Bumping version to $(NEW_VERSION)..."
	@./scripts/bump-version.sh "$(NEW_VERSION)"

# ------------------------------------------------------------
#  Release Management
# ------------------------------------------------------------

release: NEW_VERSION ?=
release: validate-env test lint
	@if [ -z "$(NEW_VERSION)" ]; then \
		echo "Error: NEW_VERSION is required"; \
		echo "Usage: make release NEW_VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "Preparing release $(NEW_VERSION)..."
	@./scripts/bump-version.sh "$(NEW_VERSION)"
	@knope prepare-release

release-dry-run:
	@echo "Performing dry run of release process..."
	@knope prepare-release --dry-run

# ------------------------------------------------------------
#  Setup & Validation Targets
# ------------------------------------------------------------

install:
	cargo fetch

validate-env: check-commands check-versions check-dev-env
	@echo "Validating Rust toolchain..."
	@rustc --version | grep -q "$(shell cat rust-toolchain 2>/dev/null || echo "$(RUST_VERSION)")" || { echo "Wrong rustc version"; exit 1; }
	@echo "Validating cargo installation..."
	@cargo --version >/dev/null 2>&1 || { echo "cargo is required but not installed"; exit 1; }
	@echo "Environment validation complete"

check-commands: COMMANDS ?= rustup npm pre-commit docker python3
check-commands:
	@for cmd in $(COMMANDS); do \
		if ! command -v $$cmd >/dev/null 2>&1; then \
			echo >&2 "$$cmd is not installed. Please install $$cmd and try again."; \
			exit 1; \
		fi \
	done

check-network: NETWORK ?= testnet
check-network:
	@if [ "$(NETWORK)" != "mainnet" ] && [ "$(NETWORK)" != "testnet" ]; then \
		echo "Error: NETWORK must be either 'mainnet' or 'testnet'"; \
			exit 1; \
	fi

check-versions:
	@echo "Checking required tool versions..."
	@echo "$(shell rustc --version)"
	@echo "$(shell cargo --version)"
	@echo "node: $(shell node --version)"
	@echo "npm: $(shell npm --version)"

check-dev-env:
	@if [ ! -f .env ]; then \
		echo "Warning: .env file not found. Copying from .env.example..."; \
		cp .env.example .env; \
	fi

setup: COMMANDS := rustup npm pre-commit
setup: check-commands check-versions check-dev-env
	./scripts/setup.sh

# ------------------------------------------------------------
#  Development Targets
# ------------------------------------------------------------

dev-watch:
	cargo watch -- cargo run

ci: lint test coverage audit

clean: clean/build clean/docker

clean/build:
	cargo clean
	rm -rf target/
	rm -rf node_modules/

cleanup_artifacts: REPO_OWNER ?= fuellabs
cleanup_artifacts: REPO_NAME ?= data-systems
cleanup_artifacts: DAYS_TO_KEEP ?= 15
cleanup_artifacts:
	@echo "Running artifact cleanup..."
	@./scripts/cleanup_artifacts.sh $(REPO_OWNER) $(REPO_NAME) $(DAYS_TO_KEEP)

# ------------------------------------------------------------
#  Docker Commands
# ------------------------------------------------------------

NETWORK ?= testnet
NETWORKS = mainnet testnet
PROFILE ?= all
PROFILES = all dev nats fuel monitoring indexer logging
DOCKER_COMPOSE = ./scripts/set_envs.sh && docker compose -f docker/docker-compose.yml --env-file .env

# Helper functions to validate Docker environment and execute commands
define check_docker_env
	@if ! docker compose version > /dev/null 2>&1; then \
		echo "Error: Docker Compose is not installed"; \
		exit 1; \
	fi
	@if [ -z "$(NETWORK)" ]; then \
		echo "Error: NETWORK variable is not set"; \
		exit 1; \
	fi
endef

# Helper function to execute docker commands with consistent parameters
define docker_cmd
	$(call check_docker_env)
	NETWORK=$(1) PORT=$(2) TELEMETRY_PORT=$(3) $(DOCKER_COMPOSE) --profile $(4) $(5)
endef

# Define rules for network-only, profile-only, and network-profile combinations
define profile_rules
# Original profile rules (without network)
start/$(2) stop/$(2) restart/$(2) clean/$(2): PROFILE = $(2)
start/$(2): start
stop/$(2): stop
restart/$(2): restart
clean/$(2): clean/docker

# Network-specific rules (defaults to 'all' or 'dev' profile)
ifeq ($(filter all dev,$(PROFILE)),$(PROFILE))
start-$(1) stop-$(1) restart-$(1) clean-$(1): NETWORK = $(1)
start-$(1): PROFILE = $(PROFILE)
start-$(1): start
stop-$(1): stop
restart-$(1): restart
clean-$(1): clean/docker

start-$(1)/$(2) stop-$(1)/$(2) restart-$(1)/$(2) clean-$(1)/$(2): NETWORK = $(1)
start-$(1)/$(2): PROFILE = $(2)
start-$(1)/$(2): start
stop-$(1)/$(2): stop
restart-$(1)/$(2): restart
clean-$(1)/$(2): clean/docker
endif
endef

# Generate rules for all profiles without network
$(foreach p,$(PROFILES),$(eval $(call profile_rules,,$(p))))
# Generate rules for all network-profile combinations
$(foreach n,$(NETWORKS),$(foreach p,$(PROFILES),$(eval $(call profile_rules,$(n),$(p)))))

start: NETWORK ?= testnet
start: PORT ?= 4000
start: TELEMETRY_PORT ?= 8080
start: PROFILE ?= all
start:
	$(call docker_cmd,$(NETWORK),$(PORT),$(TELEMETRY_PORT),$(PROFILE),up -d)

stop: NETWORK ?= testnet
stop: PORT ?= 4000
stop: TELEMETRY_PORT ?= 8080
stop: PROFILE ?= all
stop:
	$(call docker_cmd,$(NETWORK),$(PORT),$(TELEMETRY_PORT),$(PROFILE),down)

restart: stop start

clean/docker: stop
	$(call docker_cmd,$(NETWORK),$(PORT),$(TELEMETRY_PORT),$(PROFILE),down -v --rmi all --remove-orphans)

# ------------------------------------------------------------
#  Publisher Run Commands (Local Development)
# ------------------------------------------------------------

PUBLISHER_SCRIPT := ./scripts/run_publisher.sh

# Define how to run the publisher script
publisher_%: EXTRA_ARGS ?=
publisher_%:
	@network=$$(echo "$**" | cut -d'-' -f2) && \
	$(PUBLISHER_SCRIPT) --network $$network --mode $$mode --port $(PORT) --telemetry-port $(TELEMETRY_PORT) $(if $(EXTRA_ARGS),--extra-args "$(EXTRA_ARGS)")

# Publisher commands for different networks and modes
run-mainnet-dev: check-network publisher_mainnet-dev        ## Run publisher in mainnet dev mode
run-mainnet-profiling: check-network publisher_mainnet-profiling  ## Run publisher in mainnet profiling mode
run-testnet-dev: check-network publisher_testnet-dev        ## Run publisher in testnet dev mode
run-testnet-profiling: check-network publisher_testnet-profiling  ## Run publisher in testnet profiling mode

# Generic publisher command using environment variables
run-publisher: NETWORK ?= testnet
run-publisher: MODE ?= dev
run-publisher: PORT ?= 4000
run-publisher: TELEMETRY_PORT ?= 8080
run-publisher: EXTRA_ARGS ?=
run-publisher: check-network
	@$(PUBLISHER_SCRIPT) --network $(NETWORK) --mode $(MODE) --port $(PORT) --telemetry-port $(TELEMETRY_PORT) $(if $(EXTRA_ARGS),--extra-args "$(EXTRA_ARGS)")

# ------------------------------------------------------------
#  Testing
# ------------------------------------------------------------

test-watch: PROFILE ?= dev
test-watch:
	cargo watch -x "test --profile $(PROFILE)"

test: PACKAGE ?= all
test: PROFILE ?= dev
test:
	@if [ "$(PACKAGE)" = "all" ]; then \
		cargo nextest run --cargo-profile $(PROFILE) --workspace --color always --locked --no-tests=pass && \
		cargo test --profile $(PROFILE) --doc --workspace; \
	else \
		cargo nextest run --cargo-profile $(PROFILE) -p $(PACKAGE) --color always --locked --no-tests=pass && \
		cargo test --profile $(PROFILE) --doc -p $(PACKAGE); \
	fi

# coverage:
# 	RUSTFLAGS="-Z threads=8" cargo +$(RUST_NIGHTLY_VERSION) tarpaulin --config ./tarpaulin.toml

bench:
	cargo bench -p data-parser -p nats-publisher -p bench-consumers

# ------------------------------------------------------------
#  Formatting & Linting
# ------------------------------------------------------------

check:
	cargo check --all-targets --all-features

fmt: fmt-cargo fmt-rust fmt-prettier fmt-markdown

lint: check lint-cargo lint-rust lint-clippy lint-prettier lint-markdown lint-machete

fmt-cargo:
	cargo sort -w

fmt-rust:
	cargo +$(RUST_NIGHTLY_VERSION) fmt -- --color always

fmt-prettier:
	pnpm prettier:fix

fmt-markdown:
	pnpm md:fix

lint-cargo:
	cargo sort --check --workspace

lint-rust:
	cargo +$(RUST_NIGHTLY_VERSION) fmt --all --check -- --color always

lint-clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

lint-prettier:
	pnpm prettier:validate

lint-markdown:
	pnpm md:lint

lint-machete:
	cargo machete --skip-target-dir

# ------------------------------------------------------------
#  Audit
# ------------------------------------------------------------

audit:
	cargo audit

audit-fix-test:
	cargo audit fix --dry-run

audit-fix:
	cargo audit fix

# ------------------------------------------------------------
#  Build & Documentation
# ------------------------------------------------------------

build:
	cargo build --release

docs: doc
	@echo "Generating additional documentation..."
	@cargo doc --no-deps --document-private-items
	@cargo doc --workspace --no-deps

docs-serve: docs
	@echo "Serving documentation on http://localhost:8000"
	@python3 -m http.server 8000 --directory target/doc

# ------------------------------------------------------------
#  Load Testing
# ------------------------------------------------------------

load-test:
	cargo run -p load-tester -- --network testnet --max-subscriptions 10 --step-size 1

# ------------------------------------------------------------
#  Benchmarking
# ------------------------------------------------------------

bench:
	cargo bench -p data-parser -p nats-publisher -p bench-consumers

# ------------------------------------------------------------
#  Local cluster (Tilt)
# ------------------------------------------------------------

# Default values for minikube resources
TILTFILE ?= ./Tiltfile
CLUSTER_MODE ?= full
MINIKUBE_DISK_SIZE ?= 50000mb
MINIKUBE_MEMORY ?= 8000mb

# Minikube and K8s setup commands
minikube_%:
	@./cluster/scripts/$*_minikube.sh "$(MINIKUBE_DISK_SIZE)" "$(MINIKUBE_MEMORY)"

k8s_setup:
	@./cluster/scripts/setup_k8s.sh

helm_setup:  ## Update Helm dependencies
	@echo "Updating Helm dependencies..."
	cd cluster/charts/fuel-local && helm dependency update
	cd cluster/charts/fuel-nats && helm dependency update
	cd cluster/charts/fuel-streams-publisher && helm dependency update

cluster_setup: minikube_setup k8s_setup helm_setup  ## Setup both minikube and kubernetes configuration

# Define common cluster operation steps
define cluster_operation
	@echo "Running cluster with mode: $(1)"
	@./scripts/set_envs.sh
	@./cluster/scripts/gen_env_secret.sh
	@CLUSTER_MODE=$(1) tilt --file ${TILTFILE} $(2) $(ARGS)
endef

cluster_up_%:
	$(call cluster_operation,$*,up)

cluster_down_%:
	$(call cluster_operation,$*,down)

cluster_reset_%: cluster_down_$* cluster_up_$*
	@echo "Reset cluster in $* mode"

cluster_up: cluster_up_full
cluster_down: cluster_down_full
cluster_reset: cluster_down_full cluster_up_full

# ------------------------------------------------------------
#  Websocket
# ------------------------------------------------------------

ws:
	websocat -v ws://127.0.0.1:5000

# ------------------------------------------------------------
#  Help
# ------------------------------------------------------------

help:
	@echo "Available commands:"
	@echo ""
	@echo "Core Commands:"
	@echo "  all                  - Show this help message"
	@echo "  build                - Build the project in release mode"
	@echo "  clean                - Clean all artifacts"
	@echo "  clean/build          - Clean build artifacts"
	@echo "  clean/docker         - Clean docker resources"
	@echo "  install              - Install project dependencies"
	@echo "  setup                - Run the setup script"
	@echo ""
	@echo "Development Workflow:"
	@echo "  dev-watch            - Run in development watch mode"
	@echo "  ci                   - Run CI checks (lint, test, coverage, audit)"
	@echo ""
	@echo "Cluster Setup:"
	@echo "  cluster_setup        - Setup both minikube and k8s configuration"
	@echo "  minikube_setup       - Setup minikube with required addons"
	@echo "  k8s_setup            - Setup kubernetes configuration"
	@echo ""
	@echo "Version Control:"
	@echo "  version              - Show current version"
	@echo "  bump-version         - Bump version (NEW_VERSION=X.Y.Z required)"
	@echo "  release              - Prepare a new release (NEW_VERSION=X.Y.Z required)"
	@echo "  release-dry-run      - Perform a dry run of the release process"
	@echo ""
	@echo "Testing:"
	@echo "  test                 - Run tests"
	@echo "  test-watch           - Run tests in watch mode"
	@echo "  coverage             - Generate test coverage"
	@echo "  bench                - Run benchmarks"
	@echo ""
	@echo "Code Quality:"
	@echo "  fmt                  - Format all code (cargo, rust, prettier, markdown)"
	@echo "  fmt-cargo            - Format Cargo.toml files"
	@echo "  fmt-rust             - Format Rust code"
	@echo "  fmt-prettier         - Format with Prettier"
	@echo "  fmt-markdown         - Format markdown files"
	@echo "  lint                 - Run all linters"
	@echo "  lint-cargo           - Lint Cargo.toml files"
	@echo "  lint-rust            - Lint Rust code"
	@echo "  lint-clippy          - Run Clippy"
	@echo "  lint-prettier        - Lint with Prettier"
	@echo "  lint-markdown        - Lint markdown files"
	@echo "  check                - Run cargo check"
	@echo ""
	@echo "Documentation:"
	@echo "  docs                 - Generate documentation"
	@echo "  docs-serve           - Serve documentation locally"
	@echo ""
	@echo "Security:"
	@echo "  audit                - Run security audit"
	@echo "  audit-fix            - Fix security issues"
	@echo "  audit-fix-test       - Test fixing security issues"
	@echo ""
	@echo "Environment Validation:"
	@echo "  validate-env         - Validate development environment"
	@echo "  check-commands       - Check required commands are installed"
	@echo "  check-versions       - Check tool versions"
	@echo "  check-dev-env        - Check development environment"
	@echo "  check-network        - Validate network selection"
	@echo ""
	@echo "Docker Operations:"
	@echo "  start                - Start containers"
	@echo "  stop                 - Stop containers"
	@echo "  restart              - Restart containers"
	@echo "  start/<profile>      - Start specific profile ($(PROFILES))"
	@echo "  stop/<profile>       - Stop specific profile"
	@echo "  restart/<profile>    - Restart specific profile"
	@echo "  clean/<profile>      - Clean specific profile"
	@echo ""
	@echo "Network Operations:"
	@echo "  start-mainnet        - Start mainnet configuration"
	@echo "  start-testnet        - Start testnet configuration"
	@echo "  stop-mainnet         - Stop mainnet configuration"
	@echo "  stop-testnet         - Stop testnet configuration"
	@echo "  start-<network>/<profile> - Start specific network/profile combination"
	@echo "  stop-<network>/<profile>  - Stop specific network/profile combination"
	@echo ""
	@echo "Publisher Commands:"
	@echo "  run-publisher        - Run publisher with current network and mode"
	@echo "  run-mainnet-dev      - Run publisher in mainnet dev mode"
	@echo "  run-mainnet-profiling- Run publisher in mainnet profiling mode"
	@echo "  run-testnet-dev      - Run publisher in testnet dev mode"
	@echo "  run-testnet-profiling- Run publisher in testnet profiling mode"
	@echo ""
	@echo "Environment Variables:"
	@echo "  NETWORK              - Network to use (mainnet/testnet)"
	@echo "  PORT                 - Port to use (default: 4000)"
	@echo "  MODE                 - Mode to run in (dev/profiling)"
	@echo "  EXTRA_ARGS           - Additional arguments to pass to the publisher"
	@echo "  NEW_VERSION          - Version number for bump-version and release commands"
	@echo ""
	@echo "Available Profiles: $(PROFILES)"
	@echo "Available Networks: $(NETWORKS)"
	@echo "Available Modes: $(MODES)"
