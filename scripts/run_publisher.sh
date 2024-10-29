#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# ------------------------------
# Function to Display Usage
# ------------------------------
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --network     : Specify the network (mainnet|testnet)"
    echo "                  Default: testnet"
    echo "  --mode        : Specify the run mode:"
    echo "                  dev  - Run using 'cargo run -p'"
    echo "                  profiling - Build with profiling and execute binary"
    echo "                  Default: profiling"
    echo "  --port        : Specify the port number"
    echo "                  Default: 4000"
    echo "  --extra-args  : Optional additional arguments to append (in quotes)"
    echo ""
    echo "Examples:"
    echo "  $0                                               # Runs with all defaults"
    echo "  $0 --network mainnet                            # Runs mainnet with default settings"
    echo "  $0 --port 4001                                  # Runs on port 4001"
    echo "  $0 --network mainnet --port 4001 --mode dev     # Custom network, port, and mode"
    echo "  $0 --network mainnet --extra-args \"--use-elastic-log\" # With extra arguments"
    exit 1
}

# ------------------------------
# Parse Arguments
# ------------------------------
NETWORK="testnet"
MODE="profiling"
PORT="4000"
EXTRA_ARGS=""

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
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

# Validate port number
if ! [[ "$PORT" =~ ^[0-9]+$ ]] || [ "$PORT" -lt 1 ] || [ "$PORT" -gt 65535 ]; then
    echo "Error: Invalid port number specified. Must be between 1 and 65535."
    usage
fi

# Print the configuration being used
echo "Configuration:"
echo "  Network: $NETWORK"
echo "  Mode: $MODE"
echo "  Port: $PORT"
if [ -n "$EXTRA_ARGS" ]; then
    echo "  Extra Arguments: $EXTRA_ARGS"
fi
echo ""

# ------------------------------
# Function to Load Environment Variables
# ------------------------------
load_env() {
    if [ -f .env ]; then
        # Export all variables from .env, ignoring comments and empty lines
        export "$(grep -v '^#' .env | xargs)"
    else
        echo "Error: .env file not found. Please create a .env file with the necessary variables."
        exit 1
    fi
}

# ------------------------------
# Function to Validate Network Argument
# ------------------------------
validate_network() {
    if [[ "$NETWORK" != "mainnet" && "$NETWORK" != "testnet" ]]; then
        echo "Error: Invalid network specified. Choose either 'mainnet' or 'testnet'."
        usage
    fi
}

# ------------------------------
# Function to Validate Mode Argument
# ------------------------------
validate_mode() {
    if [[ "$MODE" != "dev" && "$MODE" != "profiling" ]]; then
        echo "Error: Invalid mode specified. Choose either 'dev' or 'profiling'."
        usage
    fi
}

# ------------------------------
# Function to Validate Required Variables
# ------------------------------
validate_vars() {
    local network_upper
    network_upper=$(echo "$NETWORK" | tr '[:lower:]' '[:upper:]')

    local REQUIRED_VARS=(
        "KEYPAIR"
        "${network_upper}_RELAYER"
        "${network_upper}_RELAYER_V2_LISTENING_CONTRACTS"
        "${network_upper}_RELAYER_DA_DEPLOY_HEIGHT"
        "${network_upper}_RESERVED_NODES"
        "${network_upper}_SYNC_HEADER_BATCH_SIZE"
        "${network_upper}_RELAYER_LOG_PAGE_SIZE"
    )

    # Check if required variables exist
    for VAR in "${REQUIRED_VARS[@]}"; do
        if [ -z "${!VAR}" ]; then
            echo "Error: ${VAR} is not set in the .env file."
            exit 1
        fi
    done
}

# ------------------------------
# Main Script Execution
# ------------------------------

# Validations
load_env
validate_network
validate_mode
validate_vars

# Function to get network-specific environment variable
get_network_var() {
    local base_var
    local network_upper
    local var_name

    base_var=$1
    network_upper=$(echo "$NETWORK" | tr '[:lower:]' '[:upper:]')
    var_name="${network_upper}_${base_var}"
    echo "${!var_name}"
}

# Define common arguments (changed to array)
COMMON_ARGS=(
    "fuel-core" "run"
    "--enable-relayer"
    "--keypair" "${KEYPAIR}"
    "--relayer" "$(get_network_var "RELAYER")"
    "--ip=0.0.0.0"
    "--port" "${PORT}"
    "--peering-port" "30333"
    "--utxo-validation"
    "--poa-instant" "false"
    "--enable-p2p"
    "--sync-header-batch-size" "$(get_network_var "SYNC_HEADER_BATCH_SIZE")"
    "--relayer-log-page-size=$(get_network_var "RELAYER_LOG_PAGE_SIZE")"
    "--sync-block-stream-buffer-size" "30"
)

# Network specific arguments (changed to array)
if [ "$NETWORK" == "mainnet" ]; then
    NETWORK_ARGS=(
        "--service-name" "fuel-mainnet-node"
        "--db-path" "./mnt/db-mainnet"
        "--snapshot" "./chain"
        "--bootstrap-nodes" "$(get_network_var "RESERVED_NODES")"
        "--relayer-v2-listening-contracts=$(get_network_var "RELAYER_V2_LISTENING_CONTRACTS")"
        "--relayer-da-deploy-height=$(get_network_var "RELAYER_DA_DEPLOY_HEIGHT")"
    )
else
    NETWORK_ARGS=(
        "--service-name" "fuel-testnet-node"
        "--db-path" "./mnt/db-testnet"
        "--snapshot" "./chain"
        "--bootstrap-nodes" "$(get_network_var "RESERVED_NODES")"
        "--relayer-v2-listening-contracts=$(get_network_var "RELAYER_V2_LISTENING_CONTRACTS")"
        "--relayer-da-deploy-height=$(get_network_var "RELAYER_DA_DEPLOY_HEIGHT")"
    )
fi

# Execute based on mode (updated to use arrays)
if [ "$MODE" == "dev" ]; then
    echo "Running in development mode for $NETWORK"
    cargo run -p fuel-streams-publisher -- "${COMMON_ARGS[@]}" "${NETWORK_ARGS[@]}" "${EXTRA_ARGS}"
else
    echo "Building with --profile=profiling to use samply and running for $NETWORK"
    cargo build --profile profiling --package fuel-streams-publisher
    samply record ./target/release/fuel-streams-publisher "${COMMON_ARGS[@]}" "${NETWORK_ARGS[@]}" "${EXTRA_ARGS}"
fi
