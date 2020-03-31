#!/bin/bash
set -euxo pipefail

(
cd native
cargo clippy
cargo fmt -- --check
)
