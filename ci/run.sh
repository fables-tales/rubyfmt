#!/bin/bash
set -ex

make build/rubyformat.rb

RES=$(echo "puts 'a'" | ruby build/rubyformat.rb)

if [ "$RES" != "puts(\"a\")" ]
then
    exit 1
fi
./scripts/lint.sh
./scripts/test.sh
./scripts/rspec_stress_test.sh
