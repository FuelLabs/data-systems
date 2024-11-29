#!/bin/bash

# Load environment variables
source .env

# Generate the YAML configuration
cat << EOF > cluster/charts/fuel-local/values-publisher-env.yaml
fuel-streams-publisher:
  secrets:
    RELAYER: "${RELAYER:-}"
    KEYPAIR: "${KEYPAIR:-}"
    NATS_ADMIN_PASS: "${NATS_ADMIN_PASS:-}"
EOF

echo "Generated values-publisher-env.yaml with environment variables"
