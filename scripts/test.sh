#!/bin/bash
set -ex

test_folder() {
    for file in `ls $1/*_expected.rb`
    do
        time ruby --disable=gems src/rubyfmt.rb `echo $file | sed s/expected/actual/` > /tmp/out.rb
        diff -u /tmp/out.rb $file
        if [[ $? -ne 0 ]]
        then
            echo "got diff"
            exit 1
        fi
    done
}

f_md5() {
    if [[ -z `which md5sum` ]]
    then
        md5
    else
        md5sum
    fi
}

STRING_LITERALS_EXPECTED=`ruby string_literals_stress_test.rb | f_md5`
STRING_LITERALS_ACTUAL=`ruby --disable=gems src/rubyfmt.rb string_literals_stress_test.rb | ruby | f_md5`
if [[ $STRING_LITERALS_EXPECTED != $STRING_LITERALS_ACTUAL ]]
then
    echo "string literals are broken"
    exit 1
fi

test_folder fixtures/

RUBY_VERSION=$(ruby -v | grep -o "[0-9].[0-9]" | head -n 1)
echo $RUBY_VERSION

for dir in $(find fixtures -type d -name '2.*')
do
    fixture_version=$(basename "$dir")
    if [[ $(echo "$fixture_version<=$RUBY_VERSION" | bc -l) -ne 0 ]]
    then
        test_folder $dir
    fi
done
