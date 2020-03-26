#!/bin/bash
set -ex

./script/tests/test_string_literals.sh
./script/tests/test_methods.sh
./script/tests/test_fixtures.sh
./script/tests/test_rspec_stress.sh
