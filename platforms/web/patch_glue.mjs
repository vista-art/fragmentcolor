#!/usr/bin/env node
// Patch the wasm-bindgen glue: guard Uint8Array construction against a detached
// ArrayBuffer, make a class argument of the wrong type throw a TypeError, hand wgpu
// `undefined` where WebGPU resolves an error scope or an adapter with `null`, and export
// the WindowTarget class under its old CanvasTarget name. Exits with 1, leaving the file
// as it was, when the TypeError patch finds no `_assertClass` or the null patch no call.
// Usage: node platforms/web/patch_glue.mjs [path/to/fragmentcolor.js]
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const targetArg = process.argv[2] || resolve(__dirname, 'pkg', 'fragmentcolor.js');
const path = resolve(targetArg);
let src = readFileSync(path, 'utf8');

// Two wasm-bindgen glue shapes have shipped since this patch was first written.
// Older releases assigned each shim directly:
//
//   imports.wbg.<fn> = function(arg0) { const ret = new Uint8Array(arg0); return ret; };
//
// Current releases inline the shim as a property of the `imports.wbg` object literal:
//
//   <fn>: function(arg0) {
//       const ret = new Uint8Array(arg0);
//       return ret;
//   },
//
// We match both. The replacement keeps the same wrapping syntax so neighboring
// commas / semicolons don't drift.

const bodyAssignment = /imports\.wbg\.(__wbg_new_[a-z0-9]+)\s*=\s*function\(arg0\)\s*\{\s*const\s+ret\s*=\s*new\s+Uint8Array\(arg0\);\s*return\s+ret;\s*\};/g;
const bodyObjectLit = /(__wbg_new_[a-z0-9]+)\s*:\s*function\(arg0\)\s*\{\s*const\s+ret\s*=\s*new\s+Uint8Array\(arg0\);\s*return\s+ret;\s*\},/g;

const guardedBody = `function(arg0) {
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
}`;

let count = 0;
src = src.replace(bodyAssignment, (_m, fn) => {
    count++;
    return `imports.wbg.${fn} = ${guardedBody};`;
});
src = src.replace(bodyObjectLit, (_m, fn) => {
    count++;
    return `${fn}: ${guardedBody},`;
});

if (count === 0) {
  console.warn(`[patch_glue] No Uint8Array shim matched in ${path}; glue format may have changed.`);
} else {
  console.log(`[patch_glue] Patched ${count} Uint8Array constructor shim(s) in ${path}`);
}

// An argument of the wrong class, such as `pass.addShader('nope')`, throws a `TypeError`,
// as Python raises one for the same call. wasm-bindgen's `_assertClass` checks every class
// argument, in release builds too, and throws a plain `Error`. A glue already patched passes
// as it is, so the patch runs twice without a change. A glue whose `_assertClass` no longer
// matches fails the build, since a wasm-bindgen upgrade would otherwise drop the TypeError.
const assertClass = /(function _assertClass\(instance, klass\) \{\s*if \(!\(instance instanceof klass\)\) \{\s*throw new )(Error|TypeError)\(/;
const thrown = assertClass.exec(src)?.[2];
if (thrown === undefined) {
    console.error(
        `[patch_glue] No _assertClass in ${path} matched ${assertClass}. ` +
        `wasm-bindgen changed its glue, so update the pattern in platforms/web/patch_glue.mjs ` +
        `to keep a class argument of the wrong type throwing a TypeError.`
    );
    process.exit(1);
}
if (thrown === 'Error') {
    src = src.replace(assertClass, '$1TypeError(');
    console.log(`[patch_glue] _assertClass throws a TypeError in ${path}`);
} else {
    console.log(`[patch_glue] _assertClass already throws a TypeError in ${path}`);
}

// `GPUDevice.popErrorScope` resolves with `null` when the scope caught no error, and
// `GPU.requestAdapter` with `null` when no adapter fits. wgpu 30 reads both results as a
// wasm-bindgen `JsOption`, which takes only `undefined` as empty, so a `null` reads as an
// error wgpu can't classify, which panics with "Unexpected error", or as an adapter.
// The glue resolves both with `undefined` in place of `null`. A glue already patched
// passes as it is, and one without either call fails the build, as `_assertClass` does.
const nullable = /const ret = (arg0\.(popErrorScope|requestAdapter)\([^)]*\));/g;
const emptied = '.then((value) => value ?? undefined);';
for (const call of ['popErrorScope', 'requestAdapter']) {
    const patched = new RegExp(`const ret = arg0\\.${call}\\([^)]*\\)\\.then\\(\\(value\\) => value \\?\\? undefined\\);`);
    const found = [...src.matchAll(nullable)].some((match) => match[2] === call);
    if (!found && !patched.test(src)) {
        console.error(
            `[patch_glue] No ${call} call in ${path} matched ${nullable}. ` +
            `wasm-bindgen changed its glue, so update the pattern in platforms/web/patch_glue.mjs ` +
            `to keep wgpu reading a null ${call} result as empty.`
        );
        process.exit(1);
    }
}
const emptiedCount = [...src.matchAll(nullable)].length;
src = src.replace(nullable, (_match, expression) => `const ret = ${expression}${emptied}`);
console.log(
    emptiedCount > 0
        ? `[patch_glue] popErrorScope and requestAdapter resolve null as undefined in ${path} (${emptiedCount} calls)`
        : `[patch_glue] popErrorScope and requestAdapter already resolve null as undefined in ${path}`
);

// The window target's JS class is `WindowTarget`. Code written for older releases
// imports it as `CanvasTarget`, so the glue and its typings export the same class
// under both names. Running the patch twice adds the alias once.
const alias = 'export { WindowTarget as CanvasTarget };';
const addAlias = (text) =>
    /export class WindowTarget\b/.test(text) && !text.includes(alias) ? `${text}\n${alias}\n` : text;

src = addAlias(src);
writeFileSync(path, src, 'utf8');

const typings = path.replace(/\.js$/, '.d.ts');
if (typings !== path && existsSync(typings)) {
    writeFileSync(typings, addAlias(readFileSync(typings, 'utf8')), 'utf8');
    console.log(`[patch_glue] Exported WindowTarget as CanvasTarget in ${path} and ${typings}`);
}
