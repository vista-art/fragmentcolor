import init, { Renderer, Shader, Pass, Frame, TextureTarget, CanvasTarget } from "fragmentcolor";
import { installInstrumentation } from './instrument.mjs';

const wasmUrl = new URL("./pkg/fragmentcolor_bg.wasm", import.meta.url);
await init(wasmUrl.href);

// Install JS-level instrumentation before running any examples or docs coverage.
installInstrumentation({ Renderer, Shader, Pass, TextureTarget, CanvasTarget });

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
  } finally {
    globalThis.__HC.currentModule = prev || null;
  }
}

// Run a smoke render but do not abort the page if it fails; continue to run generated examples
await withModule('platforms.web.healthcheck.smoke', async () => {
  const renderer = new Renderer();
  const target = await renderer.createTextureTarget([64, 64]);

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

  renderer.render(shader, target);

  const rpass = new Pass("single pass");
  rpass.addShader(shader);
  renderer.render(rpass, target);

  const frame = new Frame();
  frame.addPass(rpass);
  renderer.render(frame, target);

  // Additional API coverage for docs
  const _res = shader.get("resolution");
  const _uniforms = shader.listUniforms();
  const _keys = shader.listKeys();
});

// Auto-generated: helpers in global scope
import { exampleShader } from './helpers.mjs';
Object.assign(globalThis, { exampleShader });

// Auto-generated: run all extracted examples (after init)
await import('./generated_examples.mjs');
