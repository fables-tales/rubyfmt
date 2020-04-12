#!/bin/bash
set -ex

make
make release

export RUBYFMT_USE_RELEASE=1
./script/tests/test_string_literals.sh
./script/tests/test_methods.sh
./script/tests/test_fixtures.sh
./script/tests/test_rspec_stress.sh
./script/tests/test_thoughtbot_rails_apps.sh
