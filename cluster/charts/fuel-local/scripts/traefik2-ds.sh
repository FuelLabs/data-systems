#!/usr/bin/env bash

# Enable strict error handling
set -euo pipefail
[[ ${DEBUG:-} == true ]] && set -x

# Constants
K8S_NS="kube-system"
VALID_CONTEXTS=("microk8s" "docker-desktop" "minikube")
CURRENT_CONTEXT=$(kubectl config current-context)
SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

# Helper function to run kubectl commands
kubectl_cmd() {
    kubectl --context="${CURRENT_CONTEXT}" -n "${K8S_NS}" "$@"
}

# Log script location
echo "Running script from: ${SCRIPT_DIR}"

# Validate kubernetes context
if [[ ! " ${VALID_CONTEXTS[*]} " =~ ${CURRENT_CONTEXT} ]]; then
    echo "Error: Invalid kubernetes context '${CURRENT_CONTEXT}'"
    echo "Valid contexts are: ${VALID_CONTEXTS[*]}"
    exit 1
fi
echo "Using valid kubernetes context: ${CURRENT_CONTEXT}"

# Clean up old ingress controller resources
echo "Cleaning up old ingress controller resources..."
for resource in deployment service; do
    if kubectl_cmd get "${resource}/traefik-ingress-controller" &>/dev/null; then
        echo "Deleting ${resource}/traefik-ingress-controller..."
        kubectl_cmd delete "${resource}/traefik-ingress-controller"
    fi
done

# Apply new DaemonSet configuration
echo "Applying Traefik DaemonSet configuration..."
kubectl_cmd apply -f "${SCRIPT_DIR}/traefik2-ds.yaml"

echo "Traefik setup completed successfully"
