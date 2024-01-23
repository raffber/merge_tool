#!/bin/bash

set -eufo pipefail

curdir=$(dirname "$0")
rootdir=$(realpath "$curdir"/..)

cd "$rootdir"

docker build . -t merge-tool-agent
