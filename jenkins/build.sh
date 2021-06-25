#!/bin/bash

set -eufo pipefail

curdir=$(dirname "$0")
rootdir=$(realpath "$curdir"/..)

rm -rf "$rootdir"/cli/target

docker run -v "$rootdir":/data -w /data/cli -u "$(id -u)":"$(id -g)" merge-tool-agent cargo build --release

