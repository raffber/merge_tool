#!/bin/bash

set -euxfo pipefail

cd $(dirname "$0")
cd ..

rm -rf out
mkdir out

rm -rf target
mv /cache/windows-release-cache target
cargo build --release --target  x86_64-pc-windows-msvc
cp target/x86_64-pc-windows-msvc/release/merge_tool.exe out

rm -rf target
mv /cache/linux-release-cache target
cargo build --release
cp target/release/merge_tool out
