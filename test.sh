#!/usr/bin/env bash
set -euo pipefail

./clippy.sh
cargo test --all --workspace --all-features --all-targets "$@"
