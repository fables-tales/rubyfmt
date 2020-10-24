#!/bin/bash
set -ex

# shellcheck source=./script/functions.sh
source "./script/functions.sh"

test_fixtures_folder "fixtures/small"
test_fixtures_folder "fixtures/large"
