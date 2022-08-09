#!/bin/bash
set -ex

source ./script/functions.sh

ARRAY_LITERALS_EXPECTED=$(ruby ci/array_literals_stress_test.rb | f_md5)
ARRAY_LITERALS_ACTUAL=$(f_rubyfmt < ci/array_literals_stress_test.rb | ruby | f_md5)
if [[ "$ARRAY_LITERALS_EXPECTED" != "$ARRAY_LITERALS_ACTUAL" ]]
then
    echo "array literals are broken"
    exit 1
fi
