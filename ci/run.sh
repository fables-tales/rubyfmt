#!/bin/bash
set -ex

bundle exec ruby ci/set_check_status.rb "pending"
./test.sh
RESULT=$?

if [[ $RESULT == 0 ]]
then
    bundle exec ruby ci/set_check_status.rb "success"
else
    bundle exec ruby ci/set_check_status.rb "failure"
fi
