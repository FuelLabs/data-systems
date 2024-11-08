#!/bin/bash

# Set default network if not provided
NETWORK=${NETWORK:-"testnet"}
NETWORK_UPPER="$(echo "$NETWORK" | tr '[:lower:]' '[:upper:]')"

# ------------------------------
# Function to Load Environment Variables
# ------------------------------
load_env() {
    if [ -f .env ]; then
        # Read the .env file line by line, ignoring comments and empty lines
        while IFS= read -r line || [ -n "$line" ]; do
            # Skip comments and empty lines
            [[ $line =~ ^[[:space:]]*# ]] && continue
            [[ -z "$line" ]] && continue
            # Export each variable
            export "$line"
        done < .env
    else
        echo "Error: .env file not found. Please create a .env file with the necessary variables."
        exit 1
    fi
}

load_env

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
