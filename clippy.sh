#!/usr/bin/env bash
set -euo pipefail

# If the user passes "fix" or "--fix" as any argument, include --fix for cargo clippy.
FIX_FLAG=""
for arg in "$@"; do
	case "$arg" in
		fix|--fix)
			FIX_FLAG="--fix"
			;;
	esac
done

cargo fmt --all
cargo clippy --all --workspace --all-features --all-targets $FIX_FLAG
