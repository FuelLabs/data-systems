#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Load environment variables with defaults
PORT=${PORT:-9004}
MODE=${MODE:-dev}
EXTRA_ARGS=${EXTRA_ARGS:-""}

# ------------------------------
# Function to Display Usage
# ------------------------------
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --mode         : Specify the run mode (dev|profiling)"
    echo "  --port         : Port number for the API server (default: 9004)"
    echo "  --extra-args   : Optional additional arguments to append (in quotes)"
    echo ""
    echo "Examples:"
    echo "  $0                                              # Runs with all defaults"
    echo "  $0 --mode dev --port 8080                       # Custom port"
    echo "  $0 --mode dev --extra-args '\"--use-metrics\"'  # Enable metrics"
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
echo "→ Mode: ${MODE:-dev}"
echo "→ API Port: ${PORT:-9004}"
if [ -n "$EXTRA_ARGS" ]; then
    echo "→ Extra Arguments: $EXTRA_ARGS"
fi

echo -e "==========================================\n"

# Define common arguments
COMMON_ARGS=(
    "--port" "${PORT:-9003}"
)

# Execute based on mode
if [ "${MODE:-dev}" == "dev" ]; then
    cargo run -p sv-api -- "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
else
    cargo build --profile profiling --package sv-api
    samply record ./target/profiling/sv-api "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
fi
