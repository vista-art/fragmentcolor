#!/usr/bin/env bash
set -e
set -x

if [ "$1" = "--debug" ]; then
    WASM_PACK_CONFIGURATION="--dev"
    NPM_PACKAGE_TARGET="package_debug"
else
    WASM_PACK_CONFIGURATION="--release"
    NPM_PACKAGE_TARGET="package"
fi

SOURCE_DIR=$(dirname $(readlink -f "$0"))

# Build & generate bindings with wasm-pack
wasm-pack build --target web $WASM_PACK_CONFIGURATION --out-dir $SOURCE_DIR/platforms/web/pkg

wasm-pack pack $SOURCE_DIR/platforms/web/pkg

if [ "$1" = "--publish" ]; then
    wasm-pack publish $SOURCE_DIR/platforms/web/pkg
fi
