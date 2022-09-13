#!/bin/bash

set -exufo pipefail

if [[ $# != 1 ]]; then
    echo "You need to provide a commit SHA"
    exit 1
fi

curdir=$(dirname "$0")
cd "$curdir"/..

COMMIT_SHA="$1"
RUNID=$(gh run list --json "databaseId,headSha" | jq --arg COMMIT_SHA "$COMMIT_SHA" '[.[] | select(.headSha == $COMMIT_SHA) | .databaseId][0]')

if [[ $RUNID == "null" ]]; then
    echo "Could not find a test run for the given commit SHA: $COMMIT_SHA"
    exit 1
fi

rm -rf out
mkdir -p out
gh run download $RUNID --dir out
