#!/usr/bin/env bash

set -euo pipefail

# Use environment variables provided by Tilt if available
IMAGE_NAME=${EXPECTED_IMAGE:-"fuel-streams-publisher"}
TAG=${EXPECTED_TAG:-"latest"}
DOCKERFILE="docker/fuel-streams-publisher.Dockerfile"

# Ensure we're using minikube's docker daemon if not already set
if [ -z "${DOCKER_HOST:-}" ]; then
    eval $(minikube docker-env)
fi

# Build the docker image
docker build -t ${IMAGE_NAME}:${TAG} -f ${DOCKERFILE} .
