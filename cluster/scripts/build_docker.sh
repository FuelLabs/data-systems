#!/usr/bin/env bash

# Remove the -g flag from set
set -euo pipefail

# Use environment variables provided by Tilt if available
IMAGE_NAME=${1:-"fuel-streams-publisher"}
TAG=${2:-"latest"}
DOCKERFILE="cluster/docker/${IMAGE_NAME}.Dockerfile"

# Ensure we're using minikube's docker daemon
if [[ -n "${DOCKER_HOST:-}" ]]; then
    echo "Using provided DOCKER_HOST: $DOCKER_HOST"
else
    eval $(minikube docker-env)
fi

echo "Building image ${IMAGE_NAME}:${TAG} using ${DOCKERFILE}"

# Build the docker image
docker build -t ${IMAGE_NAME}:${TAG} -f ${DOCKERFILE} .
