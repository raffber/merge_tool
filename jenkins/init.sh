#!/bin/bash

set -euf -o pipefail


mkdir /data

cd /root
apt-get update
apt-get install --yes build-essential python3-pip  gfortran python3-numpy python3-regex libudev-dev udev pkg-config npm


# rust stuff...
apt-get install --yes cargo
