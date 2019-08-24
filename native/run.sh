#!/bin/bash
set -euxo pipefail
for file in $(cat files_by_length); do ruby --disable=gems test.rb "$file"; done
