#!/bin/bash
set -ex

source ./script/functions.sh

STRING_LITERALS_EXPECTED=$(ruby ci/string_literals_stress_test.rb | f_md5)
STRING_LITERALS_ACTUAL=$(f_rubyfmt ci/string_literals_stress_test.rb | ruby | f_md5)
if [[ "$STRING_LITERALS_EXPECTED" != "$STRING_LITERALS_ACTUAL" ]]
then
    echo "string literals are broken"
    exit 1
fi
