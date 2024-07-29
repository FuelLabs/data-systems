#!/bin/bash

# Install pre-commit hooks
pre-commit install

# Install fixed nightly toolchain
rustup toolchain install nightly-2023-07-10 -c rustfmt

# Install cargo global crates
cargo install cargo-binstall
cargo binstall --no-confirm cargo-watch knope cargo-sort
