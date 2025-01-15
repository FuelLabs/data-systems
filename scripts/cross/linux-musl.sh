#!/usr/bin/env bash

apt-get update
apt-get install --assume-yes musl-tools clang pkg-config libudev-dev libssl-dev
ln -s /bin/g++ /bin/musl-g++
