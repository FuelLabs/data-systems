#!/bin/bash

# Load environment variables
source .env

# Generate the YAML configuration
cat << EOF > cluster/charts/fuel-local/values-publisher-env.yaml
fuel-streams-publisher:
  extraEnv:
    - name: RELAYER
      value: "${RELAYER:-}"
    - name: KEYPAIR
      value: "${KEYPAIR:-}"
    - name: NATS_ADMIN_PASS
      value: "${NATS_ADMIN_PASS:-}"
EOF

echo "Generated values-publisher-env.yaml with environment variables"
