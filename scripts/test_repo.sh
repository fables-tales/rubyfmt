#!/bin/bash

if [[ "$#" -ne 1 ]]
then
  >&2 echo "usage: test_repo.sh path_to_repo"
  exit 64
fi

repo_path="$1"

for file in $(git --work-tree="$repo_path" --git-dir="$repo_path/.git" ls-files | grep '\.rb$')
do
  full_path="$repo_path/$file"
  errors=$(ruby --disable=gems src/rubyfmt.rb "$full_path" 2>&1)
  if [[ "$?" -ne 0 ]]
  then
    echo "$full_path"
    echo "$errors"
  fi
done
