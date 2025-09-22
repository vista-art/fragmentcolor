import init, { Renderer, Shader, Pass, Frame, TextureTarget, CanvasTarget, Mesh, Vertex } from "fragmentcolor";
import { installInstrumentation } from './instrument.mjs';

const wasmUrl = new URL("./pkg/fragmentcolor_bg.wasm", import.meta.url);
await init(wasmUrl.href);

// Install JS-level instrumentation before running any examples or docs coverage.
installInstrumentation({ Renderer, Shader, Pass, TextureTarget, CanvasTarget });

// Parse URL params (ignore skipTexture; always test everything)
const params = new URL(globalThis.location?.href || 'http://local/').searchParams;
const SKIP_TEX = false;

// Small helper to scope module name for instrumentation and proceed even on error
async function withModule(moduleName, fn) {
  globalThis.__HC = globalThis.__HC || { currentModule: null };
  const prev = globalThis.__HC.currentModule;
  globalThis.__HC.currentModule = moduleName;
  console.log(`[begin] module=${moduleName}`);
  try {
    await fn();
    console.log(`[end] module=${moduleName} status=OK`);
  } catch (e) {
    console.log(`[end] module=${moduleName} status=FAILED error=${e?.message || String(e)}]`);
    throw e; // propagate failure so healthcheck can fail
  } finally {
    globalThis.__HC.currentModule = prev || null;
  }
}

// Run a smoke render but do not abort the page if it fails; continue to run generated examples
await withModule('platforms.web.healthcheck.smoke', async () => {
  const renderer = new Renderer();
  const canvas = document.createElement('canvas');
  canvas.width = 64; canvas.height = 64;
  const target = await renderer.createTarget(canvas);
const textureTarget = await renderer.createTextureTarget([64, 64]);

  const shader = new Shader(`
struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
}
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    const vertices = array(
        vec2( -1., -1.),
        vec2(  3., -1.),
        vec2( -1.,  3.)
    );
    return VertexOutput(vec4<f32>(vertices[in_vertex_index], 0.0, 1.0));
}
@group(0) @binding(1) var<uniform> resolution: vec2<f32>;
@fragment
fn main(_v: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
`);

  shader.set("resolution", [64.0, 64.0]);

  console.log("shader before call:", shader);
  console.log("target before call:", target);

  console.log("shader, target");
  renderer.render(shader, target);
  console.log("shader after call:", shader);
  console.log("target after call:", target);

  console.log("shader, target 2");
  renderer.render(shader, target);
  console.log("shader after call 2:", shader);
  console.log("target after call 2:", target);

  console.log("shader, target 3");
  renderer.render(shader, target);
  console.log("shader after call 3:", shader);
  console.log("target after call 3:", target);

console.log("shader, textureTarget");
  renderer.render(shader, textureTarget);

  const rpass = new Pass("single pass");
  rpass.addShader(shader);
renderer.render(rpass, target);
  renderer.render(rpass, textureTarget);

  const frame = new Frame();
  frame.addPass(rpass);
renderer.render(frame, target);
  renderer.render(frame, textureTarget);

  const res = shader.get("resolution");
  console.log(res);

let image = textureTarget.getImage();
  console.log(image);

  console.log(shader.listUniforms());
  console.log(shader.listKeys());
});

// Test texture creation and shader.set parity
await withModule('platforms.web.healthcheck.texture', async () => {
  const renderer = new Renderer();
  const target = await renderer.createTextureTarget([64, 64]);
  
  // Create 2x2 red-green-blue-white texture from raw RGBA bytes
  const pixels = new Uint8Array([
    255,0,0,255,    0,255,0,255,
    0,0,255,255,    255,255,255,255,
  ]);
  const tex = await renderer.createTextureWithSize(pixels, [2, 2]);
  console.log('Created texture from raw RGBA bytes with explicit size');
  
  // Test setting texture on shader
  const shader = new Shader(`
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> resolution: vec2<f32>;

struct VOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.,1.), vec2<f32>(2.,1.), vec2<f32>(0.,-1.));
    var out: VOut;
    out.pos = vec4<f32>(p[i], 0., 1.);
    out.uv = uv[i];
    return out;
}
@fragment
fn main(v: VOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, v.uv);
}
`);
  
  shader.set("tex", tex);
  shader.set("resolution", [64.0, 64.0]);
  console.log('Set texture on shader successfully');
  
  renderer.render(shader, target);
  const img = target.getImage();
  console.log('Rendered textured shader:', img);
});

// Mesh smoke: positions-only triangle
await withModule('platforms.web.healthcheck.mesh.basic', async () => {
  const renderer = new Renderer();
  const target = await renderer.createTextureTarget([32, 32]);
  const shader = new Shader(`
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
`);
  const pass = new Pass('mesh-basic');
  pass.addShader(shader);
  const mesh = new Mesh();
  mesh.addVertex(new Vertex([-0.5, -0.5, 0.0]));
  mesh.addVertex(new Vertex([ 0.5, -0.5, 0.0]));
  mesh.addVertex(new Vertex([ 0.0,  0.5, 0.0]));
  pass.addMesh(mesh);
  renderer.render(pass, target);
  const img = target.getImage();
  console.log('mesh.basic image bytes:', img?.length || 0);
});

// Mesh smoke: two instances with offsets
await withModule('platforms.web.healthcheck.mesh.instances', async () => {
  const renderer = new Renderer();
  const target = await renderer.createTextureTarget([32, 32]);
  const shader = new Shader(`
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,1.,0.,1.); }
`);
  const pass = new Pass('mesh-inst');
  pass.addShader(shader);
  const mesh = new Mesh();
  mesh.addVertices([
    new Vertex([-0.5, -0.5, 0.0]),
    new Vertex([ 0.5, -0.5, 0.0]),
    new Vertex([ 0.0,  0.5, 0.0]),
  ]);
  const instA = new Vertex([0.0, 0.0]).set('offset', [0.0, 0.0]).createInstance();
  const instB = new Vertex([0.25, 0.0]).set('offset', [0.25, 0.0]).createInstance();
  mesh.addInstances([instA, instB]);
  pass.addMesh(mesh);
  renderer.render(pass, target);
  const img = target.getImage();
  console.log('mesh.instances image bytes:', img?.length || 0);
});

// Auto-generated: helpers in global scope
import { exampleShader } from './helpers.mjs';
Object.assign(globalThis, { exampleShader });

// Auto-generated: run all extracted examples (after init)
// await import('./generated_examples.mjs');

// Signal success for Playwright harness if we reached this point
// Push constants smoke: solid color via var<push_constant>
await withModule('platforms.web.healthcheck.push_constant', async () => {
  const renderer = new Renderer();
  const target = await renderer.createTextureTarget([8, 8]);
  const shader = new Shader(`
struct PC { color: vec4<f32> };
var<push_constant> pc: PC;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return pc.color; }
`);
  shader.set("pc.color", [1.0, 0.0, 0.0, 1.0]);
  renderer.render(shader, target);
  const img = target.getImage();
  if (!(img && img.length >= 4)) throw new Error('push-constant image empty');
  const px = [img[0], img[1], img[2], img[3]];
  const expect = [255, 0, 0, 255];
  if (px[0] !== expect[0] || px[1] !== expect[1] || px[2] !== expect[2] || px[3] !== expect[3]) {
    throw new Error(`unexpected pixel ${px} != ${expect}`);
  }
});

console.log('Headless JS render completed successfully');
