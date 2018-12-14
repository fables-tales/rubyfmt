#!/bin/bash
set -ex

bundle exec ruby ci/check_start.rb
./test.sh
RESULT=$?

if [[ $RESULT == 0 ]]
then
    bundle exec ruby ci/check_succeed.rb
else
    bundle exec ruby ci/check_fail.rb
fi
