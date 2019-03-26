#!/bin/bash
set -ex

cp -r fixtures/2.5 fixtures/2.6
for file in `ls fixtures/*_expected.rb` `ls fixtures/$(ruby -v | grep -o '\d\.\d')/*_expected.rb`
do
    time ruby --disable=gems src/rubyfmt.rb `echo $file | sed s/expected/actual/` > /tmp/out.rb
    git diff --no-index /tmp/out.rb $file
done
