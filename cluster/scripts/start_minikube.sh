#!/usr/bin/env bash

[[ $DEBUG = true ]] && set -x
set -euo pipefail

# Default values for resources
DEFAULT_DISK_SIZE='50000mb'
DEFAULT_MEMORY='20000mb'

# Get parameters with defaults
DISK_SIZE=${1:-$DEFAULT_DISK_SIZE}
MEMORY=${2:-$DEFAULT_MEMORY}

# Start minikube with specified resources
minikube start \
    --driver=docker \
    --disk-size="$DISK_SIZE" \
    --memory="$MEMORY" \
    --cpus 8 \
    --insecure-registry registry.dev.svc.cluster.local:5000

# Display minikube status
echo -e "\n\033[1;33mMinikube Status:\033[0m"
minikube status
