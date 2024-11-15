#!/usr/bin/env bash

K8S_NS="kube-system"
VALID_CONTEXTS=("microk8s" "docker-desktop" "minikube")
CURRENT_CONTEXT=$(kubectl config current-context)
SCRIPT_DIR=$(dirname $0)

if [[ "${VALID_CONTEXTS[@]}" =~ "${CURRENT_CONTEXT}" ]]; then
    echo "K8S context valid: ${CURRENT_CONTEXT}"
else
    echo "K8S invalid context: ${CURRENT_CONTEXT}"
    exit 1
fi

# delete the old ingress deployment/service since now we use a DaemonSet for Linux/Mac
if [[ $(kubectl --context="${CURRENT_CONTEXT}" -n ${K8S_NS} get deployment/traefik-ingress-controller 2> /dev/null) ]]; then
    kubectl --context="${CURRENT_CONTEXT}" -n ${K8S_NS} delete deployment/traefik-ingress-controller
fi
if [[ $(kubectl --context="${CURRENT_CONTEXT}" -n ${K8S_NS} get service/traefik-ingress-controller 2> /dev/null) ]]; then
    kubectl --context="${CURRENT_CONTEXT}" -n ${K8S_NS} delete service/traefik-ingress-controller
fi

kubectl --context="${CURRENT_CONTEXT}" -n ${K8S_NS} apply -f "$SCRIPT_DIR/traefik2-ds.yaml"
