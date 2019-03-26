#!/bin/bash

set -ex

temp=$(mktemp -q)
EXPECTED_FILENAME="${1//_actual_/_expected}"

bundle exec ruby src/rubyfmt.rb "$1" "$EXPECTED_FILENAME" > "$temp" && mv "$temp" "$EXPECTED_FILENAME"
git diff "$EXPECTED_FILENAME"
echo "do you want to commit [Y/n]"
read -rn1 ans
if [ "$ans" == "y" ]
then
  git add "$EXPECTED_FILENAME"
else
  git checkout "$EXPECTED_FILENAME"
fi
