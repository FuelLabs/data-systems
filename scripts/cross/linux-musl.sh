#!/usr/bin/env bash

apt-get update
apt-get install --assume-yes musl-tools clang
ln -s /bin/g++ /bin/musl-g++
