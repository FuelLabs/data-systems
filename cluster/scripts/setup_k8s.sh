#!/usr/bin/env bash

[[ $DEBUG = true ]] && set -x
set -euo pipefail

# Parse command line arguments
NAMESPACE="${1:-fuel-streams}"  # Use first argument, default to "fuel-streams" if not provided

# Configure namespace and context
echo -e "\n\033[1;33mConfiguring ${NAMESPACE} namespace and context:\033[0m"

# Check if namespace exists
if kubectl get namespace ${NAMESPACE} &>/dev/null; then
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
