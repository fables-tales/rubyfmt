#!/bin/bash
set -exou pipefail

GIT_ROOT=$(git rev-parse --show-toplevel)

git grep -l "bin\/bash" | xargs shellcheck -x -f gcc --source-path "$GIT_ROOT"
