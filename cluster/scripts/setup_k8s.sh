#!/usr/bin/env bash

[[ $DEBUG = true ]] && set -x
set -euo pipefail

# Configure fuel-local namespace and context
echo -e "\n\033[1;33mConfiguring fuel-local namespace and context:\033[0m"

# Check if namespace exists
if kubectl get namespace fuel-local &>/dev/null; then
    echo "Namespace fuel-local already exists"
else
    echo "Creating namespace fuel-local..."
    kubectl create namespace fuel-local
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
if [ "$CURRENT_NAMESPACE" != "fuel-local" ]; then
    echo "Setting current namespace to fuel-local..."
    kubectl config set-context --current --cluster=minikube --namespace=fuel-local
else
    echo "Context namespace is already set to fuel-local"
fi

# Verify context configuration
echo -e "\n\033[1;33mVerifying cluster context:\033[0m"
kubectl config get-contexts
