#!/bin/bash

set -euxfo pipefail

cd $(dirname "$0")
cd ..

version=$(egrep -m 1 "^version" Cargo.toml | cut -d "=" -f 2 | sed 's/[" ]//g')
changelog_version=$(grep -m 1 -oE '## \[.*?\]' CHANGELOG.md | sed -e 's/[# \[]//g' -e 's/\]//g')

if [[ $version != $changelog_version ]]; then
    echo "Invalid version in changelog"
    exit 1
fi
