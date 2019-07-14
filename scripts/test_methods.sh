#!/bin/bash
set -ex

source ./scripts/functions.sh

METHODS_EXPECTED=$(ruby ci/methods_stress_test.rb | f_md5)
METHODS_ACTUAL=$(ruby --disable=gems build/rubyfmt.rb ci/methods_stress_test.rb | ruby | f_md5)
if [[ "$METHODS_EXPECTED" != "$METHODS_ACTUAL" ]]
then
    echo "methods literals are broken"
    exit 1
fi
