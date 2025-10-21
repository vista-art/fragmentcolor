#!/usr/bin/env node
// Patch wasm-bindgen glue to guard Uint8Array construction against detached ArrayBuffer
// Usage: node platforms/web/patch_glue.mjs [path/to/fragmentcolor.js]
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const targetArg = process.argv[2] || resolve(__dirname, 'pkg', 'fragmentcolor.js');
const path = resolve(targetArg);
let src = readFileSync(path, 'utf8');

// Regex to find the specific wasm-bindgen shim that constructs a Uint8Array from an ArrayBuffer
// Pattern is tolerant to whitespace variations
const re = /imports\.wbg\.(__wbg_new_[a-z0-9]+)\s*=\s*function\(arg0\)\s*\{\s*const\s+ret\s*=\s*new\s+Uint8Array\(arg0\);\s*return\s+ret;\s*\};/g;

const replacementFor = (fn) => `imports.wbg.${fn} = function(arg0) {
    try {
        return new Uint8Array(arg0);
    } catch (err) {
        try {
            const memBytes = (typeof wasm !== 'undefined' && wasm && wasm.memory && wasm.memory.buffer) ? wasm.memory.buffer.byteLength : -1;
            if (memBytes > 0) {
                return new Uint8Array(wasm.memory.buffer);
            }
        } catch {}
        return new Uint8Array(0);
    }
};`;

let count = 0;
src = src.replace(re, (m, fn) => { count++; return replacementFor(fn); });

if (count === 0) {
  console.warn(`[patch_glue] No Uint8Array shim matched in ${path}; glue format may have changed.`);
} else {
  console.log(`[patch_glue] Patched ${count} Uint8Array constructor shim(s) in ${path}`);
}

writeFileSync(path, src, 'utf8');
