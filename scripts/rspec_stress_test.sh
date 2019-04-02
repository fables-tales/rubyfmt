#!/bin/bash
set -ex

mkdir -p /tmp/rubyfmt
if [ -z "${GITHUB_REF+x}" ]
then
    echo "not on github"
else
    rm -rf /tmp/rubyfmt/rspec-core
fi
ls /tmp/rubyfmt/rspec-core/lib || git clone --depth=1 https://github.com/rspec/rspec-core /tmp/rubyfmt/rspec-core

(
cd /tmp/rubyfmt/rspec-core || exit
git reset --hard
bundle
)

FILES=$(find /tmp/rubyfmt/rspec-core/lib -type f | grep -i '\.rb$')
for FN in $FILES
do
    echo "running rubyfmt on $FN"
    ruby --disable=gems src/rubyfmt.rb "$FN" > /tmp/rubyfmt/this_one.rb
    ruby --disable=gems src/rubyfmt.rb /tmp/rubyfmt/this_one.rb > "$FN"
done
(
cd /tmp/rubyfmt/rspec-core || exit
bundle exec rspec
git reset --hard
)

# refmt.rb replaces rubyfmt.rb
ruby --disable=gems src/rubyfmt.rb src/rubyfmt.rb > /tmp/rubyfmt/refmt.rb

FILES=$(find /tmp/rubyfmt/rspec-core/lib -type f | grep -i '\.rb$')
for FN in $FILES
do
    echo "running rubyfmt on $FN"
    ruby --disable=gems /tmp/rubyfmt/refmt.rb "$FN" > //tmp/rubyfmt/this_one.rb
    ruby --disable=gems /tmp/rubyfmt/refmt.rb //tmp/rubyfmt/this_one.rb > "$FN"
done

(
cd /tmp/rubyfmt/rspec-core || exit
bundle exec rspec
git reset --hard
)
