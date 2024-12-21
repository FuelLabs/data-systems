#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# ------------------------------
# Function to Display Usage
# ------------------------------
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --mode         : Specify the run mode (dev|profiling)"
    echo "  --port     : Port number for the API server (default: 9003)"
    echo "  --nats-url     : NATS URL (default: nats://localhost:4222)"
    echo "  --s3-enabled   : Enable S3 integration (default: true)"
    echo "  --jwt-secret   : Secret key for JWT authentication (default: secret)"
    echo "  --use-metrics  : Enable metrics (flag)"
    echo "  --extra-args   : Optional additional arguments to append (in quotes)"
    echo ""
    echo "Examples:"
    echo "  $0                                         # Runs with all defaults"
    echo "  $0 --mode dev --port 8080             # Custom port"
    echo "  $0 --mode dev --use-metrics               # Enable metrics"
    exit 1
}

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --mode)
            MODE="$2"
            shift 2
            ;;
        --port)
            PORT="$2"
            shift 2
            ;;
        --nats-url)
            NATS_URL="$2"
            shift 2
            ;;
        --s3-enabled)
            S3_ENABLED="$2"
            shift 2
            ;;
        --jwt-secret)
            JWT_SECRET="$2"
            shift 2
            ;;
        --use-metrics)
            USE_METRICS="true"
            shift
            ;;
        --extra-args)
            EXTRA_ARGS="$2"
            shift 2
            ;;
        --help)
            usage
            ;;
        *)
            echo "Error: Unknown parameter passed: $1" >&2
            usage
            ;;
    esac
done

# Load environment variables with defaults
PORT=${PORT:-9003}
NATS_URL=${NATS_URL:-nats://localhost:4222}
S3_ENABLED=${S3_ENABLED:-true}
JWT_SECRET=${JWT_SECRET:-secret}
USE_METRICS=${USE_METRICS:-false}
MODE=${MODE:-dev}

# ------------------------------
# Load Environment
# ------------------------------
source ./scripts/set_env.sh

# Print the configuration being used
echo -e "\n=========================================="
echo "⚙️ Configuration"
echo -e "=========================================="

# Runtime Configuration
echo "Runtime Settings:"
echo "→ Mode: ${MODE:-dev}"
echo "→ API Port: ${PORT:-9003}"
echo "→ NATS URL: ${NATS_URL:-"nats://localhost:4222"}"
echo "→ S3 Enabled: ${S3_ENABLED:-true}"
echo "→ JWT Secret: ${JWT_SECRET:-secret}"
echo "→ Use Metrics: ${USE_METRICS:-false}"
if [ -n "$EXTRA_ARGS" ]; then
    echo "→ Extra Arguments: $EXTRA_ARGS"
fi

echo -e "==========================================\n"

# Define common arguments
COMMON_ARGS=(
    "--port" "${PORT:-9003}"
    "--nats-url" "${NATS_URL:-"nats://localhost:4222"}"
    "--jwt-secret" "${JWT_SECRET:-secret}"
)

# Add boolean flags if enabled
if [ "${S3_ENABLED:-true}" = "true" ]; then
    COMMON_ARGS+=("--s3-enabled")
fi

if [ "${USE_METRICS:-false}" = "true" ]; then
    COMMON_ARGS+=("--use-metrics")
fi

# Execute based on mode
if [ "${MODE:-dev}" == "dev" ]; then
    cargo run -p sv-webserver -- "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
else
    cargo build --profile profiling --package sv-webserver
    samply record ./target/profiling/sv-webserver "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
fi
