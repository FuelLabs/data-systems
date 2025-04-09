#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Default mode
MODE=${MODE:-dev}

# ------------------------------
# Function to Display Usage
# ------------------------------
usage() {
    echo "Usage: $0 --mode <dev|profiling> [additional arguments]"
    echo "Options:"
    echo "  --mode : Specify the run mode (dev|profiling)"
    echo "           Default: dev"
    echo "  Additional arguments are passed directly to sv-dune (see sv-dune --help for details)."
    echo ""
    echo "Examples:"
    echo "  $0 --mode dev                            # Run in dev mode with defaults"
    echo "  $0 --mode dev --network mainnet          # Run in dev mode on mainnet"
    exit 1
}

# Parse only the --mode argument
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --mode)
            MODE="$2"
            shift 2
            ;;
        --help)
            usage
            ;;
        *)
            # Stop parsing and treat remaining args as a single set
            break
            ;;
    esac
done

# Store all remaining arguments as-is (post --mode)
ARGS=("$@")

# ------------------------------
# Load Environment
# ------------------------------
source ./scripts/set_env.sh

# Print the configuration being used
echo -e "\n=========================================="
echo "⚙️ Configuration"
echo -e "=========================================="
echo "Runtime Settings:"
echo "→ Mode: ${MODE}"
echo "→ Additional Arguments: ${ARGS[*]:-None}"
echo -e "==========================================\n"

# Execute based on mode
if [ "${MODE}" == "dev" ]; then
    cargo run -p sv-dune -- "${ARGS[@]}"
else
    cargo build --profile profiling --package sv-dune
    samply record ./target/profiling/sv-dune "${ARGS[@]}"
fi
