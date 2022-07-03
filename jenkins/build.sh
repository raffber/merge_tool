#!/bin/bash

set -eufo pipefail

curdir=$(dirname "$0")
rootdir=$(realpath "$curdir"/..)

rm -rf "$rootdir"/target

docker run -v "$rootdir":/data -w /data -u "$(id -u)":"$(id -g)" merge-tool-agent cargo build --release

