# ------------------------------------------------------------
#  Variables
# ------------------------------------------------------------

# Get version using backticks instead of Make's shell function
version := `cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "fuel-streams") | .version'`

# Constants
rust_nightly_version := "nightly-2024-11-06"
rust_version := "1.81.0"

# ------------------------------------------------------------
#  Settings
# ------------------------------------------------------------

# Allow overriding the working directory
set working-directory := "."

# ------------------------------------------------------------
#  Default Target
# ------------------------------------------------------------

default:
    @just --list

# ------------------------------------------------------------
#  Setup & Validation Targets
# ------------------------------------------------------------

install:
    cargo fetch

validate-env: (check-commands "rustup npm pre-commit docker python3") check-versions check-dev-env
    @echo "Validating Rust toolchain..."
    @rustc --version | grep -q "$(cat rust-toolchain 2>/dev/null || echo "{{rust_version}}")" || { echo "Wrong rustc version"; exit 1; }
    @echo "Validating cargo installation..."
    @cargo --version >/dev/null 2>&1 || { echo "cargo is required but not installed"; exit 1; }
    @echo "Environment validation complete"

check-commands commands="rustup npm pre-commit docker python3":
    #!/usr/bin/env bash
    for cmd in {{commands}}; do
        if ! command -v $cmd >/dev/null 2>&1; then
            echo >&2 "$cmd is not installed. Please install $cmd and try again."
            exit 1
        fi
    done

check-network network="testnet":
    #!/usr/bin/env bash
    if [ "{{network}}" != "mainnet" ] && [ "{{network}}" != "testnet" ]; then
        echo "Error: network must be either 'mainnet' or 'testnet'"
        exit 1
    fi

check-versions:
    @echo "Checking required tool versions..."
    @echo "$(rustc --version)"
    @echo "$(cargo --version)"
    @echo "node: $(node --version)"
    @echo "npm: $(npm --version)"

check-dev-env:
    #!/usr/bin/env bash
    if [ ! -f .env ]; then
        echo "Warning: .env file not found. Copying from .env.example..."
        cp .env.example .env
    fi

setup: (check-commands "rustup npm pre-commit") check-versions check-dev-env
    ./scripts/setup.sh

create-env:
    @./scripts/create_env.sh

# ------------------------------------------------------------
#  Version Management
# ------------------------------------------------------------

version:
    @echo "Current version: {{version}}"

bump-version new_version="":
    #!/usr/bin/env bash
    if [ -z "{{new_version}}" ]; then
        echo "Error: new_version is required"
        echo "Usage: just bump-version new_version=X.Y.Z"
        exit 1
    fi
    echo "Bumping version to {{new_version}}..."
    cargo set-version --workspace "$1"
    cargo update --workspace
    just fmt

release new_version="" dry_run="": (validate-env) test lint
    #!/usr/bin/env bash
    just bump-version {{new_version}}
    args=$(if [ "{{dry_run}}" = "true" ]; then echo "--dry-run"; else echo ""; fi)
    knope prepare-release $args

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

cleanup-artifacts repo_owner="fuellabs" repo_name="data-systems" days_to_keep="15":
    @echo "Running artifact cleanup..."
    @./scripts/cleanup_artifacts.sh {{repo_owner}} {{repo_name}} {{days_to_keep}}

# ------------------------------------------------------------
#  Testing
# ------------------------------------------------------------

test-watch profile="dev":
    cargo watch -x "test --profile {{profile}}"

test package="all" profile="dev":
    #!/usr/bin/env bash
    if [ "{{package}}" = "all" ]; then
        cargo nextest run --cargo-profile {{profile}} --workspace --color always --locked --no-tests=pass && \
        cargo test --profile {{profile}} --doc --workspace
    else
        cargo nextest run --cargo-profile {{profile}} -p {{package}} --color always --locked --no-tests=pass && \
        cargo test --profile {{profile}} --doc -p {{package}}
    fi

bench:
    cargo bench -p data-parser -p nats-publisher -p bench-consumers

helm-test:
    helm unittest -f "tests/**/*.yaml" -f "tests/*.yaml" cluster/charts/fuel-streams

# ------------------------------------------------------------
#  Formatting & Linting
# ------------------------------------------------------------

fmt: fmt-cargo fmt-rust fmt-prettier fmt-markdown

fmt-cargo:
    cargo sort -w

fmt-rust:
    cargo +{{rust_nightly_version}} fmt -- --color always

fmt-prettier:
    pnpm prettier:fix

fmt-markdown:
    pnpm md:fix

lint: lint-cargo lint-rust lint-clippy lint-prettier lint-markdown lint-machete

lint-cargo:
    cargo sort --check --workspace

lint-rust:
    @cargo check --all-targets --all-features
    @cargo +{{rust_nightly_version}} fmt --all --check -- --color always

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
#  Environment Variables
# ------------------------------------------------------------

export NETWORK := env_var_or_default("NETWORK", "testnet")
export MODE := env_var_or_default("MODE", "profiling")
export PORT := env_var_or_default("PORT", "4000")
export TELEMETRY_PORT := env_var_or_default("TELEMETRY_PORT", "8080")

# ------------------------------------------------------------
#  Publisher Run Commands (Local Development)
# ------------------------------------------------------------

run-publisher network=NETWORK mode=MODE port=PORT telemetry_port=TELEMETRY_PORT extra_args="": (check-network network)
    @./scripts/run_publisher.sh \
        --network {{network}} \
        --mode {{mode}} \
        $(if [ ! -z "{{port}}" ]; then echo "--port {{port}}"; fi) \
        $(if [ ! -z "{{telemetry_port}}" ]; then echo "--telemetry-port {{telemetry_port}}"; fi) \
        $(if [ ! -z "{{extra_args}}" ]; then echo "--extra-args \"{{extra_args}}\""; fi)

run-mainnet-dev: (run-publisher "mainnet" "dev")
run-mainnet-profiling: (run-publisher "mainnet" "profiling")
run-testnet-dev: (run-publisher "testnet" "dev")
run-testnet-profiling: (run-publisher "testnet" "profiling")

# ------------------------------------------------------------
#  Docker Compose
# ------------------------------------------------------------

run-docker-compose command="":
    @./scripts/set_env.sh
    @docker compose -f cluster/docker/docker-compose.yml --env-file .env {{command}}

start-nats: (run-docker-compose "up -d")
stop-nats: (run-docker-compose "down")
restart-nats: (run-docker-compose "restart")
clean-nats: (run-docker-compose "down -v --rmi all --remove-orphans")

# ------------------------------------------------------------
#  Local cluster (Tilt)
# ------------------------------------------------------------

minikube-setup disk_size="50000mb" memory="12000mb":
    @./cluster/scripts/setup_minikube.sh "{{disk_size}}" "{{memory}}"

minikube-start disk_size="50000mb" memory="12000mb":
    #!/usr/bin/env bash
    echo "Starting minikube with disk-size={{disk_size}}, memory={{memory}}..."
    minikube start \
        --driver=docker \
        --disk-size="{{disk_size}}" \
        --memory="{{memory}}" \
        --cpus 8 \
        --insecure-registry registry.dev.svc.cluster.local:5000
    echo -e "\n\033[1;33mMinikube Status:\033[0m"
    minikube status

minikube-delete:
    @echo "Deleting minikube..."
    @minikube delete

k8s-setup namespace="fuel-streams":
    @echo "Setting up k8s..."
    @./cluster/scripts/setup_k8s.sh {{namespace}}

helm-setup:
    @cd cluster/charts/fuel-streams && helm dependency update
    @cd cluster/charts/fuel-streams-publisher && helm dependency update

cluster-setup: minikube-setup k8s-setup helm-setup

pre-cluster:
    @./scripts/set_env.sh
    @./cluster/scripts/gen_env_secret.sh

run-tilt-command command mode="full":
    CLUSTER_MODE={{mode}} tilt --file ./Tiltfile {{command}}

# Cluster management commands
cluster-up mode="full": pre-cluster
    @just run-tilt-command "up" {{mode}}

cluster-down mode="full": pre-cluster
    @just run-tilt-command "down" {{mode}}

cluster-reset mode="full": pre-cluster
    @just run-tilt-command "reset" {{mode}}
