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

STRING_LITERALS_EXPECTED=`ruby string_literals_stress_test.rb | md5`
STRING_LITERALS_ACTUAL=`ruby --disable=gems src/rubyfmt.rb string_literals_stress_test.rb | ruby | md5`
if [[ $STRING_LITERALS_EXPECTED != $STRING_LITERALS_ACTUAL ]]
then
    echo "string literals are broken"
    exit 1
fi

test_folder fixtures/

RUBY_VERSION=$(ruby -v | grep -o "[0-9].[0-9]" | head -n 1)
echo $RUBY_VERSION
if [[ `echo "2.5<=$RUBY_VERSION" | bc -l` -ne 0 ]]
then
    test_folder fixtures/2.5
fi

if [[ `echo "2.6<=$RUBY_VERSION" | bc -l` -ne 0 ]]
then
    test_folder fixtures/2.6
fi
