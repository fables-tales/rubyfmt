#!/bin/bash
set -ex

source ./script/functions.sh
git status ci/string_literals_stress_test.rb

cat ci/string_literals_stress_test.rb | f_md5
ruby -e 'p File.read("ci/string_literals_stress_test.rb").tap { |x| p x.encoding }.unpack("c*")'

f_rubyfmt ci/string_literals_stress_test.rb
exit 1

#STRING_LITERALS_EXPECTED=$(ruby ci/string_literals_stress_test.rb | f_md5)
#STRING_LITERALS_ACTUAL=$(f_rubyfmt ci/string_literals_stress_test.rb | ruby | f_md5)
#if [[ "$STRING_LITERALS_EXPECTED" != "$STRING_LITERALS_ACTUAL" ]]
#then
#    echo "string literals are broken"
#    exit 1
#fi
