#!/bin/bash
set -ex

curl https://raw.githubusercontent.com/tric/trick2018/master/01-kinaba/entry.rb > /tmp/kinaba.rb
ruby src/rubyfmt.rb /tmp/kinaba.rb > /tmp/kinaba_out.rb
ruby /tmp/kinaba_out.rb
ruby src/rubyfmt.rb /tmp/kinaba_out.rb > /tmp/kinaba_out_2.rb
ruby /tmp/kinaba_out_2.rb
