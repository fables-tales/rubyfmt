#!/bin/bash
REPO_BASE=$(git rev-parse --show-toplevel)

# Prefer colordiff if installed
DIFF_BINARY=$( command -v colordiff || echo "diff" )

f_md5() {
    if command -v md5sum >/dev/null
    then
        md5sum | sed 's/[ \t]*-//'
    else
        md5
    fi
}

# shellcheck disable=SC2120
f_rubyfmt() {
    "${REPO_BASE}/target/release/rubyfmt-main" "$@"
}

diff_files() {
    IDEMPOTENCY=$1
    ACTUAL=$2
    EXPECTED=$3

    if ! $DIFF_BINARY -u "$EXPECTED" "$ACTUAL"
    then
        if [[ $IDEMPOTENCY == "i" ]]
        then
            echo "got idempotent diff"
        else
            echo "got diff between formated formatted actual and expected"
        fi
        exit 1
    fi
}
