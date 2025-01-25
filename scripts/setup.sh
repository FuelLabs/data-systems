#!/bin/bash

# Install pre-commit hooks
pre-commit install

# Install PNPM as package manager for NodeJS if it doesn't exist
if ! command -v pnpm &> /dev/null; then
    npm install -g pnpm
fi

pnpm install

# Install fixed nightly toolchain
rustup toolchain install nightly-2024-11-06 -c rustfmt

install_cmd="cargo binstall --force --no-confirm"

# Install cargo global crates
cargo install cargo-binstall
$install_cmd cargo-tarpaulin
$install_cmd samply
$install_cmd cargo-watch
$install_cmd typos-cli
$install_cmd cargo-nextest --secure

# Binstall does not support --features
cargo install cargo-audit --locked --features=fix --force
cargo install release-plz --locked
cargo install taplo-cli --locked

# Check Helm and install helm-unittest plugin
if ! command -v helm &> /dev/null; then
    echo "Warning: Helm is not installed. Please install Helm first."
else
    echo "Installing Helm unittest plugin..."
    helm plugin install https://github.com/quintush/helm-unittest
fi
