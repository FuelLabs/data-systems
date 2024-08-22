#!/usr/bin/env bash

dpkg --add-architecture arm64
apt-get update
apt-get install --assume-yes clang-8 libclang-8-dev binutils-aarch64-linux-gnu zlib1g-dev:arm64
