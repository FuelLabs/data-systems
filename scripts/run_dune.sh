#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Load environment variables with defaults
MODE=${MODE:-dev}
NETWORK=${NETWORK:-local}
DATABASE_URL=${DATABASE_URL:-"postgresql://postgres:postgres@127.0.0.1:5432/fuel_streams?sslmode=disable"}
STORAGE_TYPE=${STORAGE_TYPE:-"File"}
STORAGE_FILE_DIR=${STORAGE_FILE_DIR:-""}
EXTRA_ARGS=${EXTRA_ARGS:-""}

# ------------------------------
# Function to Display Usage
# ------------------------------
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --mode              : Specify the run mode (dev|profiling)"
    echo "                        Default: dev"
    echo "  --network          : Network to connect to (local|testnet|mainnet|staging)"
    echo "                        Default: local"
    echo "  --db-url           : Database URL to connect to"
    echo "                        Default: postgresql://root@localhost:26257/defaultdb?sslmode=disable"
    echo "  --storage-type     : Type of storage to use (S3|File)"
    echo "                        Default: File"
    echo "  --storage-file-dir : Directory for file storage"
    echo "  --extra-args       : Optional additional arguments to append (in quotes)"
    echo ""
    echo "Examples:"
    echo "  $0                                              # Runs with all defaults"
    echo "  $0 --mode dev --network mainnet                 # Run in dev mode on mainnet"
    echo "  $0 --storage-type S3                           # Run with S3 storage"
    exit 1
}

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --mode)
            MODE="$2"
            shift 2
            ;;
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --db-url)
            DATABASE_URL="$2"
            shift 2
            ;;
        --storage-type)
            STORAGE_TYPE="$2"
            shift 2
            ;;
        --storage-file-dir)
            STORAGE_FILE_DIR="$2"
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
echo "→ Network: $NETWORK"
echo "→ Storage Type: $STORAGE_TYPE"
if [ -n "$STORAGE_FILE_DIR" ]; then
    echo "→ Storage File Directory: $STORAGE_FILE_DIR"
fi
if [ -n "$EXTRA_ARGS" ]; then
    echo "→ Extra Arguments: $EXTRA_ARGS"
fi

echo -e "==========================================\n"

# Define common arguments
COMMON_ARGS=(
    "--network" "${NETWORK}"
    "--db-url" "${DATABASE_URL}"
    "--storage-type" "${STORAGE_TYPE}"
)

# Add optional storage file dir if specified
if [ -n "$STORAGE_FILE_DIR" ]; then
    COMMON_ARGS+=("--storage-file-dir" "${STORAGE_FILE_DIR}")
fi

# Execute based on mode
if [ "${MODE:-dev}" == "dev" ]; then
    cargo run -p sv-dune -- "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
else
    cargo build --profile profiling --package sv-dune
    samply record ./target/profiling/sv-dune "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
fi
