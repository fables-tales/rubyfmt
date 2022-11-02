#!/bin/bash
set -euxo pipefail

source "./script/functions.sh"
cargo build --release

test_syntax_error_stdin() {
    (
    cd "$(mktemp -d)"

    if echo "a 1,2,," | f_rubyfmt 
    then
        echo "rubyfmt didn't break as expected"
        exit 1
    fi
    )
}

test_syntax_error_file() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2,," > file.rb
    if ! f_rubyfmt -- file.rb
    then
        echo "rubyfmt broke when it shouldn't have"
        exit 1
    fi
    )
}

test_syntax_error_files() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2,," > file1.rb
    echo "a(1, 2)" > file2.rb
    if ! f_rubyfmt -- file1.rb file2.rb
    then
        echo "rubyfmt broke when it shouldn't have"
        exit 1
    fi
    )
}


test_syntax_error_file_fail_fast() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2,," > file.rb
    if f_rubyfmt --fail-fast -- file.rb
    then
        echo "rubyfmt didn't break as expected"
        exit 1
    fi
    )
}

test_syntax_error_files_fail_fast() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2,," > file1.rb
    echo "a(1, 2)" > file2.rb
    if f_rubyfmt --fail-fast -- file1.rb file2.rb
    then
        echo "rubyfmt didn't break as expected"
        exit 1
    fi
    )
}

test_io_error_file() {
    (
    cd "$(mktemp -d)"

    mkdir inner
    echo "a 1,2" > inner/file.rb
    chmod 0444  inner/file.rb

    if ! f_rubyfmt -- inner/file.rb
    then
        echo "rubyfmt broke when it shouldn't have"
        exit 1
    fi
    )
}

test_io_error_files() {
    (
    cd "$(mktemp -d)"

    if ! f_rubyfmt -- inner/file1.rb somefiledoesntexist.rb
    then
        echo "rubyfmt broke when it shouldn't have"
        exit 1
    fi
    )
}

test_io_error_file_fail_fast() {
    (
    cd "$(mktemp -d)" 

    if f_rubyfmt --fail-fast -- somefiledoesntexist.rb
    then
        echo "rubyfmt didn't break as expected"
        exit 1
    fi
    )
}

test_io_error_files_fail_fast() {
    (
    cd "$(mktemp -d)"

    echo "a 1,2" > file1.rb

    if f_rubyfmt --fail-fast -- file1.rb somefiledoesntexist.rb
    then
        echo "rubyfmt didn't break as expected"
        exit 1
    fi
    )
}

test_input_file_doesnt_exist() {
    (
    cd "$(mktemp -d)"

    if f_rubyfmt --fail-fast -- @doesntexist.txt
    then
        echo "rubyfmt didn't break as expected"
        exit 1
    fi
    )
}

test_syntax_error_stdin
test_syntax_error_file
test_syntax_error_files
test_syntax_error_file_fail_fast
test_syntax_error_files_fail_fast

test_io_error_file
test_io_error_files
test_io_error_file_fail_fast
test_io_error_files_fail_fast

test_input_file_doesnt_exist

