import init, { Renderer, Shader, Pass, Frame, TextureTarget, CanvasTarget } from "fragmentcolor";
import { installInstrumentation } from './instrument.mjs';

const wasmUrl = new URL("./pkg/fragmentcolor_bg.wasm", import.meta.url);
await init(wasmUrl.href);

// Install JS-level instrumentation before running any examples or docs coverage.
installInstrumentation({ Renderer, Shader, Pass, TextureTarget, CanvasTarget });

// Parse URL params (default to skipping texture-target examples for now)
const params = new URL(globalThis.location?.href || 'http://local/').searchParams;

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

// Auto-generated: helpers in global scope
import { exampleShader } from './helpers.mjs';
Object.assign(globalThis, { exampleShader });

// Auto-generated: run all extracted examples (after init)
// await import('./generated_examples.mjs');

// Signal success for Playwright harness if we reached this point
console.log('Headless JS render completed successfully');
