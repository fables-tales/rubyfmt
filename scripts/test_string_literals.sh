#!/bin/bash
set -ex

# this one is safe because functions does in fact exist
# shellcheck disable=SC1091
source ./scripts/functions.sh

STRING_LITERALS_EXPECTED=$(ruby string_literals_stress_test.rb | f_md5)
STRING_LITERALS_ACTUAL=$(ruby --disable=gems src/rubyfmt.rb string_literals_stress_test.rb | ruby | f_md5)
if [[ "$STRING_LITERALS_EXPECTED" != "$STRING_LITERALS_ACTUAL" ]]
then
    echo "string literals are broken"
    exit 1
fi
