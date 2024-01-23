#!/bin/bash

set -euxfo pipefail


cd $(dirname "$0")
cd ..

rm -rf target
mv /cache/linux-release-cache target
cargo test --release
