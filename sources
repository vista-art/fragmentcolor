#!/usr/bin/env bash
find "README.md" "Cargo.toml" "src" -type f -print -exec sh -c 'echo "\n--- Start of file: $1 ---"; cat "$1"; echo "\n--- End of file: $1 ---\n"' _ {} \; >sources.txt
