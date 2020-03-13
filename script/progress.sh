#!/bin/bash
set -x

source "./script/functions.sh"

test_folder() {
    find "$1" -name "*_expected.rb" -maxdepth 1 | while read -r file
    do
        # shellcheck disable=SC2001
        time f_rubyfmt "$(echo "$file" | sed s/expected/actual/)" > /tmp/out.rb

        if ! diff -u /tmp/out.rb "$file"
        then
            echo "got diff between formated actual and expected"
            echo "$file" >> .failure
        fi

        time f_rubyfmt "$file" > /tmp/out.rb

        if ! diff -u /tmp/out.rb "$file"
        then
            echo "got diff between formatted expected and expected (not idempotent)"
            echo "$file" >> .failure
        fi

        echo "$file" >> .success
    done
}

make
rm -f .success .failure

test_folder fixtures/small

RUBY_VERSION=$(ruby -v | grep -o "[0-9].[0-9]" | head -n 1)
echo "$RUBY_VERSION"

find fixtures -type d -name '2.*' | while read -r dir
do
    fixture_version=$(basename "$dir")
    if [[ $(echo "$fixture_version<=$RUBY_VERSION" | bc -l) -ne 0 ]]
    then
        test_folder "$dir"
    fi
done
wc -l .success
wc -l .failure
