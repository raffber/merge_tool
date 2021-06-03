#!/bin/bash

set -euf -o pipefail

cd /root
apt-get update
# rust stuff...
apt-get install --yes build-essential cargo

