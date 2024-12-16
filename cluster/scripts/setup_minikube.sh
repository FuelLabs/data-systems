#!/usr/bin/env bash

[[ $DEBUG = true ]] && set -x
set -euo pipefail

# Check if minikube is installed
if ! command -v minikube &> /dev/null; then
    echo "Installing minikube..."
    sudo curl -Lo minikube https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64 \
        && sudo chmod +x minikube \
        && sudo cp minikube /usr/local/bin/ \
        && sudo rm minikube
else
    echo "minikube is already installed"
fi

# Delete any existing minikube cluster
minikube delete

# Set disk and memory size, using defaults if not provided
DISK_SIZE=${1:-'50000mb'}
MEMORY=${2:-'12000mb'}

# Start minikube with specified resources
minikube start \
    --driver=docker \
    --disk-size="$DISK_SIZE" \
    --memory="$MEMORY" \
    --cpus 8

# Display minikube status
echo -e "\n\033[1;33mMinikube Status:\033[0m"
minikube status
