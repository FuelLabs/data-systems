#!/bin/bash

# Load environment variables
source .env

# Generate the YAML configuration
cat << EOF > cluster/charts/fuel-streams/values-secrets.yaml
localSecrets:
  enabled: true
  data:
    RELAYER: "${RELAYER:-}"
    KEYPAIR: "${KEYPAIR:-}"
EOF

echo "Generated values-secrets.yaml with environment variables"
