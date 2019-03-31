#!/bin/bash
set -ex

# this one is safe because functions does in fact exist
# shellcheck disable=SC1091
source ./scripts/functions.sh

curl https://raw.githubusercontent.com/tric/trick2018/master/01-kinaba/entry.rb > /tmp/kinaba.rb
ruby src/rubyfmt.rb /tmp/kinaba.rb > /tmp/kinaba_out.rb
ruby /tmp/kinaba_out.rb
ruby src/rubyfmt.rb /tmp/kinaba_out.rb > /tmp/kinaba_out_2.rb
ruby /tmp/kinaba_out_2.rb

echo "def sleep(n); end;" > /tmp/mame.rb
curl https://raw.githubusercontent.com/tric/trick2018/master/02-mame/entry.rb >> /tmp/mame.rb

ruby src/rubyfmt.rb /tmp/mame.rb > /tmp/mame_out.rb
MAME_EXPECTED=$(ruby /tmp/mame.rb | f_md5)
MAME_ACTUAL=$(ruby /tmp/mame_out.rb | f_md5)
if [[ "$MAME_EXPECTED" != "$MAME_ACTUAL" ]]
then
    echo "mame is broken"
    exit 1
fi
