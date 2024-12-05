#!/bin/bash

# Prompt for Infura API key
read -p "Enter your Infura API key: " infura_key

# Check if Infura API key was provided
if [ -z "$infura_key" ]; then
    echo "Error: Infura API key is required"
    exit 1
fi

# Generate keypair using fuel-core-keygen and extract the secret
echo "Generating keypair..."
keypair=$(fuel-core-keygen new --key-type peering | grep -o '"secret":"[^"]*"' | cut -d'"' -f4)

if [ -z "$keypair" ]; then
    echo "Error: Failed to generate keypair"
    exit 1
fi

# Copy .env.sample to .env and replace placeholders
cp .env.sample .env

# Use sed with a backup extension for compatibility
sed -i.bak "s/generated-p2p-secret/$keypair/" .env
sed -i.bak "s/<infura-api-key>/$infura_key/" .env

# Remove backup files created by sed
rm .env.bak

echo ".env file created successfully with generated keypair."

# Execute the set_env.sh script
./scripts/set_env.sh
