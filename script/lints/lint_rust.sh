#!/bin/bash
set -euxo pipefail

(
cargo clippy
cargo fmt -- --check
cd librubyfmt
cargo clippy
cargo fmt -- --check
)
