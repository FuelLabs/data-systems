#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# ------------------------------
# Function to Display Usage
# ------------------------------
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --mode        : Specify the run mode (dev|profiling)"
    echo "  --config-path : Specify the toml config path"
    echo "                  Default: config.toml"
    echo "  --extra-args  : Optional additional arguments to append (in quotes)"
    echo ""
    echo "Examples:"
    echo "  $0                                              # Runs with all defaults"
    echo "  $0 --config-path                                # Runs with default config.toml"
    echo "  $0 --mode dev                                   # Runs with dev mode"
    echo "  $0 --config-path ../config.toml --mode dev      # Custom config toml path and mode"
    exit 1
}

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --mode)
            MODE="$2"
            shift 2
            ;;
        --config-path)
            CONFIG_PATH="$2"
            shift 2
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
echo "→ Mode: $MODE"
if [ -n "$CONFIG_PATH" ]; then
    echo "→ Config path: $CONFIG_PATH"
fi
if [ -n "$EXTRA_ARGS" ]; then
    echo "→ Extra Arguments: $EXTRA_ARGS"
fi

# Environment Variables
echo -e "\nEnvironment Variables:"
echo "  → Use Metrics: ${USE_METRICS}"
echo "  → Use Elastic Logging: $USE_ELASTIC_LOGGING"
echo "  → AWS S3 Enabled: $AWS_S3_ENABLED"
echo "  → AWS Access Key Id: $AWS_ACCESS_KEY_ID"
echo "  → AWS Secret Access Key: $AWS_SECRET_ACCESS_KEY"
echo "  → AWS Region: $AWS_REGION"
echo "  → AWS Bucket: $AWS_S3_BUCKET_NAME"
echo "  → AWS Endpoint: $AWS_ENDPOINT_URL"
echo "  → Jwt Auth Secret: $JWT_AUTH_SECRET"
echo "  → Nats Url: $NATS_URL"
echo -e "==========================================\n"

# Define common arguments
COMMON_ARGS=(
    "--config-path" "${CONFIG_PATH}"
)

# Execute based on mode
if [ "$MODE" == "dev" ]; then
    cargo run -p fuel-streams-ws --bin ws-streamer -- "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
else
    cargo build --profile profiling --package fuel-streams-ws --bin ws-streamer
    samply record ./target/profiling/fuel-streams-ws "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
fi
