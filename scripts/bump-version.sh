#!/bin/bash

cargo set-version --workspace "$1"
cargo update --workspace
make fmt
