#!/bin/bash
set -ex

# this one is safe because functions does in fact exist
# shellcheck disable=SC1091
source ./scripts/functions.sh

RUBY_VERSION=$(ruby -v | grep -o "[0-9].[0-9]" | head -n 1)
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

if [[ $(echo "2.5<=$RUBY_VERSION" | bc -l) -ne 0 ]]
then
    (
    cd 03-tompng || exit
    bundle install
    bundle exec ruby entry.rb trick.png
    ruby "$RUBYFMT" entry.rb > entry_formatted.rb
    bundle exec ruby entry_formatted.rb trick.png
    )
fi

(
cd 04-colin || exit
cat > sample_test.rb <<EOD
$: << \`pwd\`.strip
require './entry.rb'
string_1 = "Hello world!"
string_2 = "This is not the same!"

🤔 "The two strings are equal", string_1 == string_2
EOD
COLIN_EXPECTED=$(ruby sample_test.rb | f_md5)
ruby "$RUBYFMT" entry.rb > entry_formatted.rb
mv entry_formatted.rb entry.rb
COLIN_ACTUAL=$(ruby sample_test.rb | f_md5)
if [[ "$COLIN_EXPECTED" != "$COLIN_ACTUAL" ]]
then
    echo "colin is broken"
    exit 1
fi
)

(
cd 05-tompng || exit
ruby entry.rb
TOMPNG_ACTUAL=$(f_md5 < wine_glass.stl)
ruby "$RUBYFMT" entry.rb > entry_formatted.rb
ruby entry_formatted.rb
TOMPNG_EXPECTED=$(f_md5 < wine_glass.stl)
if [[ "$TOMPNG_ACTUAL" != "$TOMPNG_EXPECTED" ]]
then
    echo "tompng (wineglass) is broken"
    exit 1
fi
)
