#!/bin/bash

# Set default network if not provided
NETWORK=${NETWORK:-"testnet"}
NETWORK_UPPER="$(echo "$NETWORK" | tr '[:lower:]' '[:upper:]')"

# Load .env file
source .env

# Map network-specific variables
RESERVED_NODES="$(eval echo "\$${NETWORK_UPPER}_RESERVED_NODES")"
RELAYER_V2_LISTENING_CONTRACTS="$(eval echo "\$${NETWORK_UPPER}_RELAYER_V2_LISTENING_CONTRACTS")"
RELAYER_DA_DEPLOY_HEIGHT="$(eval echo "\$${NETWORK_UPPER}_RELAYER_DA_DEPLOY_HEIGHT")"
RELAYER="$(eval echo "\$${NETWORK_UPPER}_RELAYER")"
SYNC_HEADER_BATCH_SIZE="$(eval echo "\$${NETWORK_UPPER}_SYNC_HEADER_BATCH_SIZE")"
RELAYER_LOG_PAGE_SIZE="$(eval echo "\$${NETWORK_UPPER}_RELAYER_LOG_PAGE_SIZE")"

# Validate required variables
REQUIRED_VARS=(
    "KEYPAIR"
    "RELAYER"
    "RESERVED_NODES"
    "RELAYER_V2_LISTENING_CONTRACTS"
    "RELAYER_DA_DEPLOY_HEIGHT"
    "SYNC_HEADER_BATCH_SIZE"
    "RELAYER_LOG_PAGE_SIZE"
)

for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        echo "Error: ${var} is not set"
        exit 1
    fi
done

# Export chain config
export CHAIN_CONFIG="$NETWORK"
