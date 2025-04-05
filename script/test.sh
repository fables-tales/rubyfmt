#!/bin/bash
set -euxo pipefail

rm -rf tmp/
source ./script/functions.sh

uname -a
cargo test --release
