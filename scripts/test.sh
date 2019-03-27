#!/bin/bash
set -ex

test_folder() {
    for file in `ls $1/*_expected.rb`
    do
        time ruby --disable=gems src/rubyfmt.rb `echo $file | sed s/expected/actual/` > /tmp/out.rb
        diff /tmp/out.rb $file
        if [[ $? -ne 0 ]]
        then
            echo "got diff"
            exit 1
        fi
    done
}

test_folder fixtures/

RUBY_VERSION=$(ruby -v | grep -o '\d\.\d')

if [[ `echo "2.5<=$RUBY_VERSION" | bc -l` -ne 0 ]]
then
    test_folder fixtures/2.5
fi

if [[ `echo "2.6<=$RUBY_VERSION" | bc -l` -ne 0 ]]
then
    test_folder fixtures/2.6
fi
