#!/usr/bin/env bash

[[ $DEBUG = true ]] && set -x
set -euo pipefail

minikube delete \
    && sudo rm -rf /usr/local/bin/minikube \
    && sudo curl -Lo minikube https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64 \
    && sudo chmod +x minikube \
    && sudo cp minikube /usr/local/bin/ \
    && sudo rm minikube \
    && minikube start --driver=docker --disk-size='50000mb' --memory='20000mb' --cpus 8 --insecure-registry registry.dev.svc.cluster.local:5000 \
    &&
    # Enabling addons: ingress, dashboard
    minikube addons enable ingress \
    && minikube addons enable registry-creds \
    && minikube addons enable registry \
    && minikube addons enable metrics-server \
    && minikube addons enable dashboard \
    &&
    # Showing enabled addons
    echo '\n\n\033[4;33m Enabled Addons \033[0m' \
    && minikube addons list | grep STATUS && minikube addons list | grep enabled \
    &&
    # Showing current status of Minikube
    echo '\n\n\033[4;33m Current status of Minikube \033[0m' && minikube status
