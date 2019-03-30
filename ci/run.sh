#!/bin/bash
set -ex
RES=`echo "puts 'a'" | ruby src/rubyfmt.rb`
if [ $RES != "puts(\"a\")"]
then
    exit 1
fi
./scripts/lint.sh
./scripts/test.sh
