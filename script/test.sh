#!/bin/bash
set -euxo pipefail

rm -rf tmp/
source ./script/functions.sh

target="${TARGET:-native}"

case "$target" in
  native)
    cargo build --release
    ;;
  aarch64-unknown-linux-gnu)
    sudo apt-get update
    sudo apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross

    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
      CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
      TARGET_CC=aarch64-linux-gnu-gcc \
      TARGET_AR=aarch64-linux-gnu-ar \
      cargo build --release --target aarch64-unknown-linux-gnu

    # Don't try to test things: even with QEMU support, it would take a long time.
    exit 0
    ;;
  *)
    echo "Unknown cross-compilation target $target"
    exit 1
    ;;
esac


export RUBYFMT_USE_RELEASE=1
uname -a
./script/tests/test_string_literals.sh
./script/tests/test_array_literals.sh
./script/tests/test_methods.sh
./script/tests/test_cli_interface.sh
./script/tests/test_c_main.sh
./script/tests/test_error_handling.sh
./script/tests/test_fixtures.sh
./script/tests/test_formatting_locks.sh
#./script/tests/test_rspec_stress.sh
