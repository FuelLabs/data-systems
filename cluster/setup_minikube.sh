#!/usr/bin/env bash

[[ $DEBUG = true ]] && set -x
set -euo pipefail

# Check if minikube is installed
if ! command -v minikube &>/dev/null; then
    echo "Installing minikube..."
    sudo curl -Lo minikube https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64 &&
        sudo chmod +x minikube &&
        sudo cp minikube /usr/local/bin/ &&
        sudo rm minikube
else
    echo "minikube is already installed"
fi

# Delete any existing minikube cluster
minikube delete

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

# Enable required addons
echo "Enabling minikube addons..."
ADDONS=(
    "registry-creds"
    "registry"
    "metrics-server"
    "dashboard"
    # "ingress"
)

for addon in "${ADDONS[@]}"; do
    echo "Enabling $addon..."
    minikube addons enable "$addon"
done

# Display enabled addons
echo -e "\n\033[1;33mEnabled Addons:\033[0m"
minikube addons list | grep -E "STATUS|enabled"

# Display minikube status
echo -e "\n\033[1;33mMinikube Status:\033[0m"
minikube status

# Check if traefik script exists
TRAEFIK_SCRIPT="./cluster/charts/fuel-local/scripts/traefik2-ds.sh"
if [ -f "$TRAEFIK_SCRIPT" ]; then
    echo -e "\n\033[1;33mSetting up traefik:\033[0m"
    $TRAEFIK_SCRIPT
else
    echo -e "\n\033[1;31mWarning: Traefik setup script not found at $TRAEFIK_SCRIPT\033[0m"
    exit 1
fi
