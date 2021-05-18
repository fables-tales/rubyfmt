#!/bin/bash
set -euxo pipefail

(
cargo clippy
cargo fmt -- --check
)
