#!/bin/bash
set -ex

source ./script/functions.sh
make target/c_main_release

METHODS_EXPECTED=$(ruby ci/methods_stress_test.rb | f_md5)
METHODS_ACTUAL=$(./target/c_main_release < ci/methods_stress_test.rb | ruby | f_md5)
if [[ "$METHODS_EXPECTED" != "$METHODS_ACTUAL" ]]
then
    echo "methods literals are broken"
    exit 1
fi
