#!/bin/bash
f_md5() {
    if command -v md5sum >/dev/null
    then
        md5sum
    else
        md5
    fi
}
