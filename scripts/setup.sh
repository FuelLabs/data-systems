#!/bin/bash

# Install pre-commit hooks
pre-commit install

# Install nightly toolchain
rustup toolchain install nightly -c rustfmt

# Install cargo global crates
cargo install cargo-binstall
cargo install cargo-audit --locked --features=fix
cargo binstall --no-confirm cargo-watch knope cargo-sort
