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
$install_cmd knope
$install_cmd cargo-sort
$install_cmd typos-cli
$install_cmd cargo-nextest --secure
$install_cmd just

# Binstall does not support --features
cargo install cargo-audit --locked --features=fix --force
