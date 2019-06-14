#!/bin/bash

set -ex

temp=$(mktemp -q)
# shellcheck disable=SC2001
EXPECTED_FILENAME=$(echo "$1" | sed s/expected/actual/)

bundle exec ruby build/rubyformat.rb "$1" "$EXPECTED_FILENAME" > "$temp" && mv "$temp" "$EXPECTED_FILENAME"
git diff "$EXPECTED_FILENAME"
echo "do you want to commit [Y/n]"
read -rn1 ans
if [ "$ans" == "y" ]
then
  git add "$EXPECTED_FILENAME"
else
  git checkout "$EXPECTED_FILENAME"
fi
