#!/bin/bash
set -euxo pipefail

source "./script/functions.sh"
cargo build --release

test_stdin_stdout() {
    (
    cd "$(mktemp -d)"

    if echo "a 1,2," | f_rubyfmt 2>out
    then
        echo "rubyfmt didn't break as expected"
        exit 1
    fi
    )
}
test_single_file_stdout() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2," > file
    if  f_rubyfmt file
    then
        echo "rubyfmt didn't break as expected"
        exit 1
    fi
    )
}

test_stdin_stdout
test_single_file_stdout
