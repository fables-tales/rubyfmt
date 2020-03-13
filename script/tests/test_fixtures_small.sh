#!/bin/bash
set -ex

source "./script/functions.sh"

make

test_fixtures_folder "fixtures/small"
