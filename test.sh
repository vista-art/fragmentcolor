#!/usr/bin/env bash
set -euo pipefail

cargo build --all --workspace --all-features --all-targets "$@"
cargo test --all --workspace --all-features --all-targets "$@"

./healthcheck.sh
