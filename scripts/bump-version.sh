#!/bin/bash

cargo set-version --locked --workspace "$1"
cargo update --workspace
make fmt
