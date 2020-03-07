#!/bin/bash
set -ex

GIT_ROOT=$(git rev-parse --show-toplevel)

# shellcheck source=./script/_test_fixtures_dir.sh
source "$GIT_ROOT/script/_test_fixtures_dir.sh"

make

test_folder "$GIT_ROOT/fixtures/large"
