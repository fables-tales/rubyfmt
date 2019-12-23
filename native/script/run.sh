#!/bin/bash
set -euxo pipefail
cargo build && touch *.c && make
success=0
fail=0
set -e
for file in $(cat files_by_length | grep -v '2.6'); do
    ruby --disable=gems test.rb "$file"
done

for file in $(cat files_by_length | grep -v '2.6'); do
    set +e;
    ruby --disable=gems test.rb "$file"
    if [[ $? -eq 0 ]]
    then
        success=$((success + 1 ))
    else
        fail=$((fail + 1))
    fi
done
echo $success
echo $fail

