#!/bin/bash
set -ex
EXPECTED_FILENAME=`echo $1 | sed s/_actual/_expected/`
bundle exec ruby src/rubyfmt.rb $1 $EXPECTED_FILENAME > $EXPECTED_FILENAME
git diff $EXPECTED_FILENAME
echo "do you want to commit [Y/n]"
read -n1 ans
if [ "$ans" == "y" ]
then
  git add $EXPECTED_FILENAME
else
  git checkout $EXPECTED_FILENAME
fi
