// WebGL2 fallback smoke: the runner deletes `navigator.gpu` before this
// module runs, so the engine must pick the GL backend. Uniform blocks whose
// WGSL size is not a multiple of 16 bytes once drew nothing here.
import init, { Renderer, Shader } from "fragmentcolor";

console.log("[webgl2] boot; navigator.gpu:", typeof navigator.gpu);

const wasmUrl = new URL("./pkg/fragmentcolor_bg.wasm", import.meta.url);
await init({ module_or_path: wasmUrl.href });

if (navigator.gpu) throw new Error("webgl2 smoke expects navigator.gpu to be absent");

const TRI = `
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
  return VOut(vec4<f32>(p[i], 0.0, 1.0));
}`;

const CASES = [
  {
    name: "f32 uniform (4 bytes)",
    source: `@group(0) @binding(0) var<uniform> k: f32;
${TRI}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(k, 0.0, 0.0, 1.0); }`,
    set: (s) => s.set("k", 1.0),
    expect: [255, 0, 0, 255],
  },
  {
    name: "two-float struct (8 bytes)",
    source: `struct G { t: f32, k: f32 };
@group(0) @binding(0) var<uniform> u: G;
${TRI}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(u.t, u.k, 0.0, 1.0); }`,
    set: (s) => { s.set("u.t", 0.0); s.set("u.k", 1.0); },
    expect: [0, 255, 0, 255],
  },
  {
    name: "mixed struct (40 bytes)",
    source: `struct Globals { resolution: vec2<f32>, mouse: vec2<f32>, drag: vec2<f32>, time: f32, zoom: f32, press: f32, pulse: f32 };
@group(0) @binding(0) var<uniform> u: Globals;
${TRI}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(u.press, u.pulse, u.zoom, 1.0); }`,
    set: (s) => { s.set("u.resolution", [64.0, 64.0]); s.set("u.zoom", 1.0); s.set("u.press", 0.0); s.set("u.pulse", 0.0); },
    expect: [0, 0, 255, 255],
  },
];

// The GL backend needs a canvas to create its context, so the adapter is
// requested through a canvas target before any offscreen target exists.
const renderer = new Renderer();
const canvas = document.createElement("canvas");
canvas.width = 8;
canvas.height = 8;
document.body.appendChild(canvas);
await renderer.createTarget(canvas);
console.log("[webgl2] canvas target ready");
const target = await renderer.createTextureTarget([8, 8]);
console.log("[webgl2] texture target ready");
let failed = 0;
for (const c of CASES) {
  try {
    const shader = new Shader(c.source);
    c.set(shader);
    renderer.render(shader, target);
    console.log(`[webgl2] rendered: ${c.name}`);
    const img = await target.getImage();
    const px = [img[0], img[1], img[2], img[3]];
    if (px.some((v, i) => Math.abs(v - c.expect[i]) > 2)) {
      throw new Error(`pixel ${px} != ${c.expect}`);
    }
    console.log(`[webgl2] ok: ${c.name}`);
  } catch (e) {
    failed++;
    console.error(`[webgl2] FAIL: ${c.name}: ${e?.message || e}`);
  }
}
if (failed) throw new Error(`${failed} webgl2 case(s) failed`);
console.log("✅ webgl2 test result: ok");
