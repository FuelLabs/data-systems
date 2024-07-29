#!/bin/bash

# Install pre-commit hooks
pre-commit install

# Install fixed nightly toolchain
rustup toolchain install nightly-2024-07-28 -c rustfmt

# Install cargo global crates
cargo install cargo-binstall
cargo binstall --no-confirm cargo-watch knope cargo-sort
