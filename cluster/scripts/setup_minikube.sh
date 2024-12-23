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

minikube addons enable metrics-server
minikube addons enable registry

# Remove existing registry proxy container if running
if docker ps -a | grep -q "minikube-registry-proxy"; then
    echo "Removing existing registry proxy container..."
    docker rm -f minikube-registry-proxy
fi

# Forward minikube registry to localhost
docker run --rm -d \
    --network=host \
    --name minikube-registry-proxy \
    alpine ash -c "apk add socat && socat TCP-LISTEN:5000,reuseaddr,fork TCP:$(minikube ip):5000"

# Display minikube status
echo -e "\n\033[1;33mMinikube Status:\033[0m"
minikube status

# Parse command line arguments
NAMESPACE="${1:-fuel-streams}" # Use first argument, default to "fuel-streams" if not provided

# Configure namespace and context
echo -e "\n\033[1;33mConfiguring ${NAMESPACE} namespace and context:\033[0m"

# Check if namespace exists
if kubectl get namespace ${NAMESPACE} &> /dev/null; then
    echo "Namespace ${NAMESPACE} already exists"
else
    echo "Creating namespace ${NAMESPACE}..."
    kubectl create namespace ${NAMESPACE}
fi

# Switch to minikube context
if ! kubectl config current-context | grep -q "minikube"; then
    echo "Switching to minikube context..."
    kubectl config use-context minikube
else
    echo "Already in minikube context"
fi

# Set namespace for current context
CURRENT_NAMESPACE=$(kubectl config view --minify --output 'jsonpath={..namespace}')
if [ "$CURRENT_NAMESPACE" != "${NAMESPACE}" ]; then
    echo "Setting current namespace to ${NAMESPACE}..."
    kubectl config set-context --current --cluster=minikube --namespace=${NAMESPACE}
else
    echo "Context namespace is already set to ${NAMESPACE}"
fi

# Verify context configuration
echo -e "\n\033[1;33mVerifying cluster context:\033[0m"
kubectl config get-contexts
