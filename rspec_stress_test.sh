#!/bin/bash
set -ex

mkdir -p tmp
ls tmp/rspec-core/lib || git clone --depth=1 https://github.com/rspec/rspec-core tmp/rspec-core

for i in {1..50}
do
    echo "formatting $i"
    FN=`find tmp/rspec-core/lib -type f | grep -i '\.rb$' | sort --random-sort | head -n 1`

    echo "running rubyfmt on $FN"

    ruby src/rubyfmt.rb $FN > tmp/out || (cd tmp/rspec-core && git reset --hard && exit 1)
    cp tmp/out $FN
done

cd tmp/rspec-core && bundle install && bundle exec rspec || git reset --hard
