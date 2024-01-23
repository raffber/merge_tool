#!/bin/bash

set -eufo pipefail

curdir=$(dirname "$0")
rootdir=$(realpath "$curdir"/..)

docker run -v "$rootdir":/data -w /data merge-tool-agent ci/test-ci.sh

