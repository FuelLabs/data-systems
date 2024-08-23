#!/bin/bash

# Load environment variables from .env file
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo ".env file not found. Please create a .env file using the '.env.sample' template and try again."
    exit 1
fi

# Function to check if environment variables are set
check_env_vars() {
    local missing_vars=0

    while IFS= read -r line; do
        # Skip empty lines and comments
        [[ -z "$line" || "$line" == \#* ]] && continue

        # Extract the key
        key=$(echo "$line" | cut -d '=' -f 1)

        # Check if the key is set in the environment
        if [ -z "${!key}" ]; then
            echo "Environment variable $key is not set."
            missing_vars=$((missing_vars + 1))
        fi
    done < ".env.sample"

    return $missing_vars
}

check_env_vars

cargo run -p fuel-streams-publisher -- \
    --service-name "Fuel Streams Publisher" \
    --ip 0.0.0.0 \
    --port 4000 \
    --peering-port 30333 \
    --db-path ./docker/db \
    --snapshot ./docker/chain-config \
    --utxo-validation \
    --poa-instant false \
    --enable-p2p \
    --keypair $KEYPAIR \
    --sync-header-batch-size $SYNC_HEADER_BATCH_SIZE \
    --enable-relayer \
    --relayer $RELAYER \
    --relayer-v2-listening-contracts $RELAYER_V2_LISTENING_CONTRACTS \
    --relayer-da-deploy-height $RELAYER_DA_DEPLOY_HEIGHT \
    --relayer-log-page-size $RELAYER_LOG_PAGE_SIZE \
    --reserved-nodes /dns4/p2p-testnet.fuel.network/tcp/30333/p2p/16Uiu2HAmDxoChB7AheKNvCVpD4PHJwuDGn8rifMBEHmEynGHvHrf
