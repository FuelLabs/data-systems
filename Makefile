# ------------------------------------------------------------
#  Variables
# ------------------------------------------------------------

# Version detection using shell command
VERSION := $(shell cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "fuel-streams") | .version')

# Constants
RUST_NIGHTLY_VERSION := nightly-2024-11-06
RUST_VERSION := 1.81.0

# ------------------------------------------------------------
#  Phony Targets
# ------------------------------------------------------------

.PHONY: install validate-env check-commands check-network check-versions \
        check-dev-env setup create-env version bump-version release dev-watch \
        clean clean-build cleanup-artifacts test-watch test bench helm-test \
        fmt fmt-cargo fmt-rust fmt-prettier fmt-markdown lint lint-cargo \
        lint-rust lint-clippy lint-prettier lint-markdown lint-machete \
        audit audit-fix-test audit-fix load-test run-publisher run-consumer \
        run-mainnet-dev run-mainnet-profiling run-testnet-dev run-testnet-profiling \
        start-nats stop-nats restart-nats clean-nats minikube-setup minikube-start \
        minikube-delete k8s-setup helm-setup cluster-setup pre-cluster \
        cluster-up cluster-down cluster-reset

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

check-commands:
	@for cmd in rustup npm pre-commit docker python3; do \
		if ! command -v $$cmd >/dev/null 2>&1; then \
			echo "$$cmd is not installed. Please install $$cmd and try again."; \
			exit 1; \
		fi \
	done

check-network:
	@if [ "$(NETWORK)" != "mainnet" ] && [ "$(NETWORK)" != "testnet" ]; then \
		echo "Error: network must be either 'mainnet' or 'testnet'"; \
		exit 1; \
	fi

check-versions:
	@echo "Checking required tool versions..."
	@echo "$$(rustc --version)"
	@echo "$$(cargo --version)"
	@echo "node: $$(node --version)"
	@echo "npm: $$(npm --version)"

check-dev-env:
	@if [ ! -f .env ]; then \
		echo "Warning: .env file not found. Copying from .env.example..."; \
		cp .env.example .env; \
	fi

setup: check-commands check-versions check-dev-env
	./scripts/setup.sh

create-env:
	@./scripts/create_env.sh

# ------------------------------------------------------------
#  Version Management
# ------------------------------------------------------------

version:
	@echo "Current version: $(VERSION)"

bump-version:
	@if [ -z "$(NEW_VERSION)" ]; then \
		echo "Error: NEW_VERSION is required"; \
		echo "Usage: make bump-version NEW_VERSION=X.Y.Z"; \
		exit 1; \
	fi
	@echo "Bumping version to $(NEW_VERSION)..."
	cargo set-version --workspace "$(NEW_VERSION)"
	cargo update --workspace
	$(MAKE) fmt

release: validate-env test lint
	@if [ -z "$(NEW_VERSION)" ]; then \
		echo "Error: NEW_VERSION is required"; \
		echo "Usage: make release NEW_VERSION=X.Y.Z [dry_run=true]"; \
		exit 1; \
	fi
	$(MAKE) bump-version NEW_VERSION=$(NEW_VERSION)
	knope prepare-release $(if $(filter true,$(dry_run)),--dry-run,)

# ------------------------------------------------------------
#  Development Targets
# ------------------------------------------------------------

dev-watch:
	cargo watch -- cargo run

clean: clean-build

clean-build:
	cargo clean
	rm -rf target/
	rm -rf node_modules/

cleanup-artifacts: REPO_OWNER="fuellabs"
cleanup-artifacts: REPO_NAME="data-systems"
cleanup-artifacts: DAYS_TO_KEEP=10
cleanup-artifacts:
	@echo "Running artifact cleanup..."
	@./scripts/cleanup_artifacts.sh $(REPO_OWNER) $(REPO_NAME) $(DAYS_TO_KEEP)

# ------------------------------------------------------------
#  Testing
# ------------------------------------------------------------

test-watch: PROFILE="all"
test-watch:
	cargo watch -x "test --profile $(PROFILE)"

test: PACKAGE="all"
test: PROFILE="dev"
test:
	@echo "Running tests for package $(PACKAGE) with profile $(PROFILE)"
	@if [ "$(PACKAGE)" = "all" ] || [ -z "$(PACKAGE)" ]; then \
		cargo nextest run --cargo-profile $(PROFILE) --workspace --color always --no-tests=pass --all-features && \
		cargo test --profile $(PROFILE) --doc --workspace --all-features; \
	else \
		cargo nextest run --cargo-profile $(PROFILE) -p $(PACKAGE) --color always --no-tests=pass --all-features && \
		cargo test --profile $(PROFILE) --doc -p $(PACKAGE) --all-features; \
	fi

bench:
	cargo bench -p data-parser

helm-test:
	helm unittest -f "tests/**/*.yaml" -f "tests/*.yaml" cluster/charts/fuel-streams

# ------------------------------------------------------------
#  Formatting & Linting
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

lint: lint-cargo lint-rust lint-clippy lint-prettier lint-markdown lint-machete

lint-cargo:
	cargo sort --check --workspace

lint-rust:
	@cargo check --all-targets --all-features
	@cargo +$(RUST_NIGHTLY_VERSION) fmt --all --check -- --color always

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
#  Load Testing & Benchmarking
# ------------------------------------------------------------

load-test:
	cargo run -p load-tester -- --network testnet --max-subscriptions 10 --step-size 1

# ------------------------------------------------------------
#  Publisher Run Commands
# ------------------------------------------------------------

run-publisher: NETWORK="testnet"
run-publisher: PACKAGE="sv-emitter"
run-publisher: MODE="dev"
run-publisher: PORT="4000"
run-publisher: TELEMETRY_PORT="8080"
run-publisher: NATS_URL="localhost:4222"
run-publisher: EXTRA_ARGS=""
run-publisher: check-network
	@./scripts/run_publisher.sh

run-publisher-mainnet-dev:
	$(MAKE) run-publisher NETWORK=mainnet MODE=dev

run-publisher-mainnet-profiling:
	$(MAKE) run-publisher NETWORK=mainnet MODE=profiling

run-publisher-testnet-dev:
	$(MAKE) run-publisher NETWORK=testnet MODE=dev

run-publisher-testnet-profiling:
	$(MAKE) run-publisher NETWORK=testnet MODE=profiling

run-consumer: NATS_CORE_URL="localhost:4222"
run-consumer: NATS_PUBLISHER_URL="localhost:4223"
run-consumer:
	cargo run --package sv-consumer --profile dev -- \
		--nats-core-url $(NATS_CORE_URL) \
		--nats-publisher-url $(NATS_PUBLISHER_URL)

# ------------------------------------------------------------
#  Streamer Run Commands
# ------------------------------------------------------------

run-streamer: check-network
	@./scripts/run_streamer.sh \
		--mode $(MODE) \
		$(if $(CONFIG_PATH),--config-path $(CONFIG_PATH),) \
		$(if $(extra_args),--extra-args "$(extra_args)",)

run-streamer-mainnet-dev:
	$(MAKE) run-streamer NETWORK=mainnet MODE=dev CONFIG_PATH=crates/fuel-streams-ws/config.toml

run-streamer-mainnet-profiling:
	$(MAKE) run-streamer NETWORK=mainnet MODE=profiling CONFIG_PATH=crates/fuel-streams-ws/config.toml

run-streamer-testnet-dev:
	$(MAKE) run-streamer NETWORK=testnet MODE=dev CONFIG_PATH=crates/fuel-streams-ws/config.toml

run-streamer-testnet-profiling:
	$(MAKE) run-streamer NETWORK=testnet MODE=profiling CONFIG_PATH=crates/fuel-streams-ws/config.toml

# ------------------------------------------------------------
#  Docker Compose
# ------------------------------------------------------------

run-docker-compose:
	@./scripts/set_env.sh
	@docker compose -f cluster/docker/docker-compose.yml --env-file .env $(COMMAND)

start-nats:
	$(MAKE) run-docker-compose COMMAND="up -d"

stop-nats:
	$(MAKE) run-docker-compose COMMAND="down"

restart-nats:
	$(MAKE) run-docker-compose COMMAND="restart"

clean-nats:
	$(MAKE) run-docker-compose COMMAND="down -v --rmi all --remove-orphans"

reset-nats: clean-nats start-nats

# ------------------------------------------------------------
#  Local cluster (Minikube)
# ------------------------------------------------------------

# Environment variables with defaults
NETWORK ?= testnet
MODE ?= profiling
PORT ?= 4000
TELEMETRY_PORT ?= 8080

minikube-setup:
	@./cluster/scripts/setup_minikube.sh "$(DISK_SIZE)" "$(MEMORY)"

minikube-start:
	@echo "Starting minikube with disk-size=$(DISK_SIZE), memory=$(MEMORY)..."
	minikube start \
		--driver=docker \
		--disk-size="$(DISK_SIZE)" \
		--memory="$(MEMORY)" \
		--cpus 8 \
		--insecure-registry registry.dev.svc.cluster.local:5000
	@echo -e "\n\033[1;33mMinikube Status:\033[0m"
	@minikube status

minikube-delete:
	@echo "Deleting minikube..."
	@minikube delete

k8s-setup:
	@echo "Setting up k8s..."
	@./cluster/scripts/setup_k8s.sh $(NAMESPACE)

helm-setup:
	@cd cluster/charts/fuel-streams && helm dependency update
	@cd cluster/charts/fuel-streams-publisher && helm dependency update

cluster-setup: minikube-setup k8s-setup helm-setup

pre-cluster:
	@./scripts/set_env.sh
	@./cluster/scripts/gen_env_secret.sh

# Cluster management commands
cluster-up: pre-cluster
	CLUSTER_MODE=$(MODE) tilt --file ./Tiltfile up

cluster-down: pre-cluster
	CLUSTER_MODE=$(MODE) tilt --file ./Tiltfile down

cluster-reset: pre-cluster
	CLUSTER_MODE=$(MODE) tilt --file ./Tiltfile reset
