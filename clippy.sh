#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all
cargo clippy --all --workspace --all-features --all-targets $@ -- -D warnings
