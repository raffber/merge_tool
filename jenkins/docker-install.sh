#!/bin/bash

set -euf -o pipefail

ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

mkdir -p /data /home /rust /cargo
chmod a+rwx /data /home /rust /cargo

cd /root
apt-get update
apt-get install --yes build-essential mingw-w64 libclang-dev curl

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

rustup install stable
rustup target add x86_64-pc-windows-gnu
rustup toolchain install stable-x86_64-pc-windows-gnu

