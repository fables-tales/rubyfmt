#!/bin/bash
set -ex

RUBY_VERSION=$(ruby -v | grep -o "[0-9].[0-9]" | head -n 1)

test_folder() {
    current_dir="$1"

    find "$current_dir" -name "*_expected.rb" -maxdepth 1 | while read -r expected_file; do
      actual_file="${expected_file//expected/actual}"

      ## Test if the formatting works as expected
      time ruby --disable=gems rubyfmt.rb "$actual_file" > /tmp/out.rb
      if ! diff -u /tmp/out.rb "$expected_file" > /tmp/diff.out
      then
        echo "got diff between formated formatted actual and expected"
        cat /tmp/diff.out
        exit 1
      fi

      ## Test if the formatting is idempotent
      time ruby --disable=gems rubyfmt.rb "$expected_file" > /tmp/out.rb
      if ! diff -u /tmp/out.rb "$expected_file" > /tmp/diff.out
      then
        echo "got diff between formatted expected and expected (not idempotent)"
        cat /tmp/diff.out
        exit 1
      fi
    done

    ## Recurse over ruby version dirs
    find "$current_dir" -type d -mindepth 1 -maxdepth 1 -name '2.' | while read -r dir
    do
        base="$(basename "$dir")"
        fixture_version=${base#"ruby-"}
        if [[ $(echo "$fixture_version<=$RUBY_VERSION" | bc -l) -ne 0 ]]
        then
            test_folder "$dir"
        fi
    done
}
