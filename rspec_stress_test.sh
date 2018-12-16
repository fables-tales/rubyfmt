#!/bin/bash
set -ex

mkdir -p tmp
ls tmp/rspec-core/lib || git clone --depth=1 https://github.com/rspec/rspec-core tmp/rspec-core
FN=`find tmp/rspec-core/lib -type f | grep -i '\.rb$' | sort --random-sort | head -n 1`

echo "running rubyfmt on $FN"

ruby src/rubyfmt.rb $FN > tmp/out
cp tmp/out $FN

cd tmp/rspec-core && bundle install && bundle exec rspec || git reset --hard
