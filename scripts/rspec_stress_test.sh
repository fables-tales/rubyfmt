#!/bin/bash
set -ex

RUBYFMT=$(pwd)/native/test.rb
make

#cd "$(mktemp -d)"
#mkdir -p tmp
#if [ -z "${GITHUB_REF+x}" ]
#then
#    echo "not on github"
#else
#    rm -rf tmp/rspec-core
#fi
#ls tmp/rspec-core/lib || git clone --depth=1 https://github.com/rspec/rspec-core tmp/rspec-core
#
cd tmp/rspec-core
git reset --hard
bundle
cd ../..

FILES=$(find tmp/rspec-core/lib -type f | grep -i '\.rb$')
for FN in $FILES
do
    echo "running rubyfmt on $FN"
    ruby --disable=gems "$RUBYFMT" "$FN" > /tmp/this_one.rb
    ruby --disable=gems "$RUBYFMT" /tmp/this_one.rb > "$FN"
    cd tmp/rspec-core
    bundle exec rspec --exclude-pattern ./spec/integration/persistence_failures_spec.rb
    git reset --hard
    cd ../../
done

## refmt.rb replaces rubyfmt.rb
#ruby --disable=gems "$RUBYFMT" "$RUBYFMT" > tmp/refmt.rb
#
#FILES=$(find tmp/rspec-core/lib -type f | grep -i '\.rb$')
#for FN in $FILES
#do
#    echo "running rubyfmt on $FN"
#    ruby --disable=gems tmp/refmt.rb "$FN" > /tmp/this_one.rb
#    ruby --disable=gems tmp/refmt.rb /tmp/this_one.rb > "$FN"
#done
#cd tmp/rspec-core
#bundle exec rspec --exclude-pattern ./spec/integration/persistence_failures_spec.rb
#git reset --hard
#cd ../../
#rm -rf tmp
