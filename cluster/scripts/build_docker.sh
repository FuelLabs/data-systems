#!/usr/bin/env bash

# Remove the -g flag from set
set -euo pipefail

# Help/Usage function
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Build a Docker image using specified parameters.

Options:
    --dockerfile     Path to Dockerfile (default: cluster/docker/sv-publisher.Dockerfile)
    --build-args     Additional Docker build arguments (optional)
    -h, --help       Show this help message

Environment variables:
    TAG             Docker image tag (default: latest)
    DOCKER_HOST     Docker daemon socket (optional)

Examples:
    $(basename "$0") --dockerfile ./Dockerfile
    $(basename "$0") --dockerfile ./Dockerfile --build-args "--build-arg KEY=VALUE"
EOF
    exit 1
}

# Show help if no arguments or help flag
if [[ $# -eq 0 ]] || [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    usage
fi

# Default values
DOCKERFILE="cluster/docker/sv-publisher.Dockerfile"
IMAGE_NAME=${EXPECTED_IMAGE:-"sv-publisher"}
TAG=${EXPECTED_TAG:-"latest"}
BUILD_ARGS=""

# Parse named arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dockerfile)
            DOCKERFILE="$2"
            shift 2
            ;;
        --build-args)
            BUILD_ARGS="$2"
            shift 2
            ;;
        *)
            echo "Error: Unknown argument '$1'"
            usage
            ;;
    esac
done

# Validate required files exist
if [[ ! -f "$DOCKERFILE" ]]; then
    echo "Error: Dockerfile not found at $DOCKERFILE"
    exit 1
fi

# Ensure we're using minikube's docker daemon
if [[ -n "${DOCKER_HOST:-}" ]]; then
    echo "Using provided DOCKER_HOST: $DOCKER_HOST"
else
    eval $(minikube docker-env)
fi

echo "Building image ${IMAGE_NAME}:${TAG} using ${DOCKERFILE}"
echo "Build args: ${BUILD_ARGS}"

# Build the docker image with build args if provided
if [[ -n "${BUILD_ARGS}" ]]; then
    docker build ${BUILD_ARGS} -t "${IMAGE_NAME}:${TAG}" -f "${DOCKERFILE}" .
else
    docker build -t "${IMAGE_NAME}:${TAG}" -f "${DOCKERFILE}" .
fi
