#!/bin/bash

# Check if a command exists using which
check_command() {
    if ! which "$1" >/dev/null 2>&1; then
        echo "$1 is not installed. Please install $1 and try again."
        exit 1
    fi
}

# Check if Rust is installed
check_command rustup
# Check if pre-commit is installed
check_command pre-commit

# Install pre-commit hooks
pre-commit install

# Install nightly toolchain
rustup toolchain install nightly -c rustfmt

# Install cargo global crates
cargo install cargo-binstall
cargo binstall --no-confirm cargo-watch knope cargo-sort
