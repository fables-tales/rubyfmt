#!/bin/bash
set -ex

for file in `ls fixtures/*_expected.rb`
do
    ruby --disable=gems run.rb `echo $file | sed s/expected/actual/` > /tmp/out.rb
    diff /tmp/out.rb $file
    if [[ $? -ne 0 ]]
    then
        echo "got diff"
        exit 1
    fi
done
