#!/bin/bash

# Load environment variables from .env file
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo ".env file not found. Please create a .env file using the '.env.sample' template and try again."
    exit 1
fi

check_env_var() {
    local var_name="$1"
    local var_value="${!var_name}"
    if [ -z "$var_value" ]; then
        echo "Environment variable $var_name is not set. Aborting."
        exit 1
    fi
}

check_env_var "INFURA_API_KEY"
check_env_var "GENERATED_P2P_SECRET"

ETH_RPC_ENDPOINT="https://sepolia.infura.io/v3/${INFURA_API_KEY}"
P2P_SECRET="${GENERATED_P2P_SECRET}"

cd ./crates/fuel-core-nats

cargo run --all-features --bin fuel-core-nats -- \
    --service-name "NATS Publisher Node" \
    --ip 0.0.0.0 \
    --port 4000 \
    --peering-port 30333 \
    --utxo-validation \
    --poa-instant false \
    --enable-p2p \
    --keypair $P2P_SECRET \
    --sync-header-batch-size=100 \
    --enable-relayer \
    --relayer ${ETH_RPC_ENDPOINT} \
    --relayer-v2-listening-contracts=0x768f9459E3339A1F7d59CcF24C80Eb4A711a01FB \
    --relayer-da-deploy-height=5791365 \
    --relayer-log-page-size=2000 \
