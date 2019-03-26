#!/bin/bash
set -ex

mkdir -p tmp
if [ -z "${GITHUB_REF+x}" ]
then
    echo "not on github"
else
    rm -rf tmp/rspec-core
fi
ls tmp/rspec-core/lib || git clone --depth=1 https://github.com/rspec/rspec-core tmp/rspec-core

cd tmp/rspec-core
git reset --hard
bundle
cd ../..

FILES=$(find tmp/rspec-core/lib -type f | grep -i '\.rb$')
for FN in $FILES
do
    echo "running rubyfmt on $FN"
    ruby --disable=gems src/rubyfmt.rb "$FN" > /tmp/this_one.rb
    ruby --disable=gems src/rubyfmt.rb /tmp/this_one.rb > "$FN"
done
cd tmp/rspec-core
bundle exec rspec
git reset --hard
cd ../../

# refmt.rb replaces rubyfmt.rb
ruby --disable=gems src/rubyfmt.rb src/rubyfmt.rb > tmp/refmt.rb

FILES=$(find tmp/rspec-core/lib -type f | grep -i '\.rb$')
for FN in $FILES
do
    echo "running rubyfmt on $FN"
    ruby --disable=gems tmp/refmt.rb "$FN" > /tmp/this_one.rb
    ruby --disable=gems tmp/refmt.rb /tmp/this_one.rb > "$FN"
done
cd tmp/rspec-core
bundle exec rspec
git reset --hard
cd ../../
