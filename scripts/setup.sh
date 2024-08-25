#!/bin/bash

# Install pre-commit hooks
pre-commit install

# Install PNPM as package manager for NodeJS if it doesn't exist
if ! command -v pnpm &> /dev/null; then
    npm install -g pnpm
fi

pnpm install

# Install fixed nightly toolchain
rustup toolchain install nightly-2024-07-28 -c rustfmt

# Install cargo global crates
cargo install cargo-binstall
cargo install cargo-tarpaulin
cargo install cargo-audit --locked --features=fix
cargo binstall --no-confirm cargo-watch knope cargo-sort typos-cli
