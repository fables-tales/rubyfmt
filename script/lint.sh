#!/bin/bash
set -exou pipefail

git grep -l "bin\/bash" | xargs shellcheck -x -f gcc
