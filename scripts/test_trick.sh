#!/bin/bash
set -ex

# this one is safe because functions does in fact exist
# shellcheck disable=SC1091
source ./scripts/functions.sh

RUBYFMT=$(pwd)/src/rubyfmt.rb

git clone https://github.com/tric/trick2018 /tmp/trick2018 || echo "already have repo"
cd /tmp/trick2018
git reset --hard
git checkout a9eb6555e0e3ba2ca8ebe0fd6be3671423f0aed4

ruby "$RUBYFMT" 01-kinaba/entry.rb > ./kinaba_out.rb
ruby ./kinaba_out.rb
ruby "$RUBYFMT" ./kinaba_out.rb > ./kinaba_out_2.rb
ruby ./kinaba_out_2.rb

echo "def sleep(n); end;" | cat - 02-mame/entry.rb  > 02-mame/fast_entry.rb

ruby "$RUBYFMT" 02-mame/fast_entry.rb > ./mame_out.rb
MAME_EXPECTED=$(ruby 02-mame/fast_entry.rb | f_md5)
MAME_ACTUAL=$(ruby ./mame_out.rb | f_md5)
if [[ "$MAME_EXPECTED" != "$MAME_ACTUAL" ]]
then
    echo "mame is broken"
    exit 1
fi
