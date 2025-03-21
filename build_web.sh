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
wasm-pack build --target web $WASM_PACK_CONFIGURATION

# Build npm package
rm -rf $SOURCE_DIR/platforms/web/dist
npm run --prefix $SOURCE_DIR/platforms/web $NPM_PACKAGE_TARGET -- --outDir $SOURCE_DIR/platforms/web/dist
