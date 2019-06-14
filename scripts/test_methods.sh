#!/bin/bash
set -ex

source ./scripts/functions.sh

METHODS_EXPECTED=$(ruby methods_stress_test.rb | f_md5)
METHODS_ACTUAL=$(ruby --disable=gems build/rubyformat.rb methods_stress_test.rb | ruby | f_md5)
if [[ "$METHODS_EXPECTED" != "$METHODS_ACTUAL" ]]
then
    echo "string literals are broken"
    exit 1
fi
