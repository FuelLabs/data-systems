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
    echo "  --mode        : Specify the run mode (dev|profiling)"
    echo "                  Default: profiling"
    echo "  --port        : Specify the port number"
    echo "                  Default: 4000"
    echo "  --telemtry-port : Specify the telemetry port number"
    echo "                  Default: 8080"
    echo "  --extra-args  : Optional additional arguments to append (in quotes)"
    echo ""
    echo "Examples:"
    echo "  $0                                              # Runs with all defaults"
    echo "  $0 --network mainnet                            # Runs mainnet with default settings"
    echo "  $0 --port 4001                                  # Runs on port 4001"
    echo "  $0 --telemetry-port 8081                        # uses telemetry port 8081"
    echo "  $0 --network mainnet --port 4001 --telemetry-port 8081 --mode dev     # Custom network, port, telemetry-port and mode"
    exit 1
}

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
source ./scripts/set_envs.sh

# Print the configuration being used
echo -e "\n=========================================="
echo "⚙️ Configuration"
echo -e "=========================================="

# Runtime Configuration
echo "Runtime Settings:"
echo "  → Network: $NETWORK"
echo "  → Mode: $MODE"
echo "  → Port: $PORT"
echo "  → Telemetry Port: $TELEMETRY_PORT"
if [ -n "$EXTRA_ARGS" ]; then
    echo "→ Extra Arguments: $EXTRA_ARGS"
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
    "--keypair" "${KEYPAIR}"
    "--relayer" "${RELAYER}"
    "--ip=0.0.0.0"
    "--service-name" "fuel-${NETWORK}-node"
    "--db-path" "./docker/db-${NETWORK}"
    "--snapshot" "./docker/chain-config/${NETWORK}"
    "--port" "${PORT}"
    "--telemetry-port" "${TELEMETRY_PORT}"
    "--peering-port" "30333"
    "--utxo-validation"
    "--poa-instant" "false"
    "--enable-p2p"
    "--sync-header-batch-size" "${SYNC_HEADER_BATCH_SIZE}"
    "--relayer-log-page-size=${RELAYER_LOG_PAGE_SIZE}"
    "--sync-block-stream-buffer-size" "30"
    "--bootstrap-nodes" "${RESERVED_NODES}"
    "--relayer-v2-listening-contracts=${RELAYER_V2_LISTENING_CONTRACTS}"
    "--relayer-da-deploy-height=${RELAYER_DA_DEPLOY_HEIGHT}"
    "--nats-url=nats://localhost:4222"
)

# Execute based on mode
if [ "$MODE" == "dev" ]; then
    cargo run -p fuel-streams-publisher -- "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
else
    cargo build --profile profiling --package fuel-streams-publisher
    samply record ./target/profiling/fuel-streams-publisher "${COMMON_ARGS[@]}" ${EXTRA_ARGS}
fi
