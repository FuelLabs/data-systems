#!/usr/bin/env bash

[[ $DEBUG = true ]] && set -x
set -euo pipefail

minikube start --driver=docker --disk-size='50000mb' --memory='20000mb' --cpus 8 --insecure-registry registry.dev.svc.cluster.local:5000 \
    &&
    # Showing current status of Minikube
    echo '\n\n\033[4;33m Current status of Minikube \033[0m' && minikube status
