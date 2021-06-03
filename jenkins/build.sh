#!/bin/bash

curdir=$(dirname "$0")
rootdir=$(realpath "$curdir"/..)

rm -rf "$rootdir"/comsrv/target

docker run -v "$rootdir":/data -w /data/cli -u "$(id -u)":"$(id -g)" merge-tool-agent cargo build --release

