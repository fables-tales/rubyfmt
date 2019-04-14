#!/bin/bash
set -ex
make build/rubyfmt.rb

./scripts/test_string_literals.sh
./scripts/test_methods.sh
./scripts/test_trick.sh
./scripts/test_fixtures.sh
