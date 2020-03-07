#!/bin/bash
set -e

lint_dir_deep() {
    current_dir="$1"

    find "$current_dir" -name "*.rb" | while read -r file; do
      actual_file="${file//expected/actual}"
      expected_file="${file//actual/expected}"

      [ -f "$actual_file" ] || (echo "$file exists but $actual_file does not" && exit 1)
      [ -f "$expected_file" ] || (echo "$file exists but $expected_file does not" && exit 1)
    done
}

RUBY_VERSION=$(ruby -v | grep -o "[0-9].[0-9]" | head -n 1)

lint_dir_deep "fixtures"
