#!/bin/bash
set -ex

lint_dir_deep() {
    current_dir="$1"

    find "$current_dir" -name "*.rb" | while read -r file; do
      actual_file="${file//_expected.rb/_actual.rb}"
      expected_file="${file//_actual.rb/_expected.rb}"

      [[ "$actual_file" == "$expected_file" ]] &&
        echo "$file does not match the naming conventions for the fixtures. The file must end in '_actual.rb' or '_expected.rb'" && exit 1
      [ -f "$actual_file" ] || (echo "$file exists but $actual_file does not" && exit 1)
      [ -f "$expected_file" ] || (echo "$file exists but $expected_file does not" && exit 1)
    done
}

lint_dir_deep "fixtures"
