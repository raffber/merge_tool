#!/bin/bash

curdir=$(dirname "$0")
rootdir=$(realpath "$curdir"/..)

docker run -v "$rootdir":/root/ws -w /root/ws/cli merge-tool-agent cargo build


