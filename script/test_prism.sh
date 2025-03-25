#!/bin/bash
set -euxo pipefail

rm -rf tmp/
source ./script/functions.sh

export RUBYFMT_PRISM=1
cargo build --release

uname -a

# TODO(reese): uncomment these as we build out Prism support
# ./script/tests/test_string_literals.sh
# ./script/tests/test_array_literals.sh
# ./script/tests/test_methods.sh
# ./script/tests/test_cli_interface.sh
# ./script/tests/test_c_main.sh
# ./script/tests/test_error_handling.sh
# ./script/tests/test_fixtures.sh
FIXTURE_NAME=numbers ./script/tests/test_fixtures.sh
# ./script/tests/test_formatting_locks.sh
