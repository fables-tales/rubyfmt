#!/bin/bash
set -e

test_folder() {
    current_dir="$1"

    ## Check that both files or neither exist
    if [ -f "$current_dir/expected.rb" ] || [ -f "$current_dir/actual.rb" ]; then
      [ -f "$current_dir/expected.rb" ] || (echo "$current_dir is missing the expected.rb file" && exit 1)
      [ -f "$current_dir/actual.rb" ] || (echo "$current_dir is missing the actual.rb file" && exit 1)

      ## Test if the formatting works as expected
      time ruby --disable=gems rubyfmt.rb "$current_dir/actual.rb" > /tmp/out.rb
      if ! diff -u /tmp/out.rb "$current_dir/expected.rb" > /tmp/diff.out
      then
        echo "got diff between formated formatted actual and expected"
        cat /tmp/diff.out
        exit 1
      fi

      ## Test if the formatting is idempotent
      time ruby --disable=gems rubyfmt.rb "$current_dir/expected.rb" > /tmp/out.rb
      if ! diff -u /tmp/out.rb "$current_dir/expected.rb" > /tmp/diff.out
      then
        echo "got diff between formatted expected and expected (not idempotent)"
        cat /tmp/diff.out
        exit 1
      fi
    fi

    ## Recurse over ruby version dirs
    find "$current_dir" -type d -mindepth 1 -maxdepth 1 -name 'ruby-' | while read -r dir
    do
        base="$(basename "$dir")"
        fixture_version=${base#"ruby-"}
        if [[ $(echo "$fixture_version<=$RUBY_VERSION" | bc -l) -ne 0 ]]
        then
            test_folder "$dir"
        fi
    done

    # Recurse over the other dirs
    find "$current_dir" -type d -mindepth 1 -maxdepth 1 -not -name 'ruby-' | while read -r dir
    do
          test_folder "$dir"
    done
}


RUBY_VERSION=$(ruby -v | grep -o "[0-9].[0-9]" | head -n 1)

if [ $# -eq 0 ]; then
    TEST_DIR="fixtures"
else
    [ -d "fixtures/$1" ] || (echo "fixtures/$1 dir does not exist" && exit 1)
    TEST_DIR="fixtures/$1"
fi

make

test_folder "$TEST_DIR"
