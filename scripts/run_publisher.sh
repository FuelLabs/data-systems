#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# ------------------------------
# Function to Display Usage
# ------------------------------
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  --network     : Specify the network (mainnet|testnet|local)"
    echo "                  Default: testnet"
    echo "  --mode        : Specify the run mode (dev|profiling)"
    echo "                  Default: profiling"
    echo "  --port        : Specify the port number"
    echo "                  Default: 4001"
    echo "  --telemetry-port : Specify the telemetry port number"
    echo "                  Default: 8080"
    echo "  --extra-args  : Optional additional arguments to append (in quotes)"
    echo "  --from-block : Specify the starting block height"
    echo "                  Default: 0"
    echo ""
    echo "Examples:"
    echo "  $0                                              # Runs with all defaults"
    echo "  $0 --network mainnet                            # Runs mainnet with default settings"
    echo "  $0 --port 9000                                  # Runs on port 9000"
    echo "  $0 --network mainnet --port 4001 --telemetry-port 8080 --mode dev     # Custom network, port, telemetry-port and mode"
    echo "  $0 --from-block 1000                           # Start from block height 1000"
    exit 1
}

# Set default values from environment variables with fallbacks
NETWORK=${NETWORK:-"testnet"}
MODE=${MODE:-"profiling"}
PORT=${PORT:-"4004"}
TELEMETRY_PORT=${TELEMETRY_PORT:-"9002"}
FROM_BLOCK=${FROM_BLOCK:-"0"}

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
        --telemetry-port)
            TELEMETRY_PORT="$2"
            shift 2
            ;;
        --extra-args)
            ARGS="$2"
            shift 2
            ;;
        --from-block)
            FROM_BLOCK="$2"
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
source ./scripts/set_env.sh NETWORK=${NETWORK}

# Print the configuration being used
echo -e "\n=========================================="
echo "⚙️ Configuration"
echo -e "=========================================="

# Runtime Configuration
echo "Runtime Settings:"
echo "  → Network: $NETWORK"
echo "  → Mode: $MODE"
echo "  → Fuel Block Consumer Port: $PORT"
echo "  → Telemetry Port: $TELEMETRY_PORT"
echo "  → From Height: $FROM_BLOCK"
if [ -n "$ARGS" ]; then
    echo "→ Extra Arguments: $ARGS"
fi

# Environment Variables
echo -e "\nEnvironment Variables:"
echo "  → Keypair: ${KEYPAIR:0:15}...${KEYPAIR: -15}"
echo "  → Relayer: $RELAYER"
echo "  → Reserved Nodes: ${RESERVED_NODES:0:50}..."
echo "  → Header Batch Size: $SYNC_HEADER_BATCH_SIZE"
echo "  → Relayer Log Page: $RELAYER_LOG_PAGE_SIZE"
echo "  → V2 Contracts: $RELAYER_V2_LISTENING_CONTRACTS"
echo "  → DA Deploy Height: $RELAYER_DA_DEPLOY_HEIGHT"
echo -e "==========================================\n"

# Define common arguments
COMMON_ARGS=(
    "--enable-relayer"
    "--service-name" "fuel-${NETWORK}-node"
    "--keypair" "${KEYPAIR}"
    "--relayer" "${RELAYER}"
    "--ip=0.0.0.0"
    "--port" "${PORT}"
    "--peering-port" "30333"
    "--db-path" "./cluster/docker/db-${NETWORK}"
    "--snapshot" "./cluster/chain-config/${NETWORK}"
    "--utxo-validation"
    "--poa-instant" "false"
    "--enable-p2p"
    "--reserved-nodes" "${RESERVED_NODES}"
    "--relayer-v2-listening-contracts=${RELAYER_V2_LISTENING_CONTRACTS}"
    "--relayer-da-deploy-height=${RELAYER_DA_DEPLOY_HEIGHT}"
    "--relayer-log-page-size=${RELAYER_LOG_PAGE_SIZE}"
    "--sync-block-stream-buffer-size" "50"
    "--max-database-cache-size" "17179869184"
    "--state-rewind-duration" "136y"
    "--request-timeout" "60s"
    "--graphql-max-complexity" "1000000000"
    # Application specific
    "--nats-url" "nats://localhost:4222"
    "--telemetry-port" "${TELEMETRY_PORT}"
    "--from-block" "${FROM_BLOCK}"
)

# Execute based on mode
if [ "$MODE" == "dev" ]; then
    cargo run -p sv-publisher -- "${COMMON_ARGS[@]}" ${ARGS}
else
    cargo build --profile profiling --package sv-publisher
    samply record ./target/profiling/sv-publisher "${COMMON_ARGS[@]}" ${ARGS}
fi
