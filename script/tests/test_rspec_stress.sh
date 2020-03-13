#!/bin/bash
set -ex
source "./script/functions.sh"


make

#cd "$(mktemp -d)"
#mkdir -p tmp
#if [ -z "${GITHUB_REF+x}" ]
#then
#    echo "not on github"
#else
#    rm -rf tmp/rspec-core
#fi
ls tmp/rspec-core/lib || git clone --depth=1 https://github.com/rspec/rspec-core tmp/rspec-core
cd tmp/rspec-core
git reset --hard
bundle
cd ../..

FILES=$(find tmp/rspec-core/lib -type f | grep -i '\.rb$')
for FN in $FILES
do
    echo "running rubyfmt on $FN"
    f_rubyfmt "$FN" > /tmp/this_one.rb
    f_rubyfmt /tmp/this_one.rb > "$FN"
done
cd tmp/rspec-core
bundle exec rspec --exclude-pattern ./spec/integration/persistence_failures_spec.rb
git reset --hard
cd ../../
