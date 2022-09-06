#!/bin/bash

set -euxfo pipefail


cd $(dirname "$0")
cd ..

cargo test --release
