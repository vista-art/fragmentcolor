import init, { Renderer, Shader, Pass, Frame } from "../pkg/fragmentcolor.js";

const wasmUrl = new URL("../pkg/fragmentcolor_bg.wasm", import.meta.url);
await init(wasmUrl.href);

// DOC: Renderer.constructor (begin)
const renderer = new Renderer();
// DOC: (end)
// DOC: Renderer.create_texture_target (begin)
const target = await renderer.createTextureTarget([64, 64]);
// DOC: (end)

// DOC: Shader.constructor (begin)
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
// DOC: (end)

// DOC: Shader.set (begin)
shader.set("resolution", [64.0, 64.0]);
// DOC: (end)

// DOC: Renderer.render (begin)
renderer.render(shader, target);
// DOC: (end)

// DOC: Pass.constructor (begin)
const rpass = new Pass("single pass");
// DOC: (end)
// DOC: Pass.add_shader (begin)
rpass.addShader(shader);
// DOC: (end)
renderer.render(rpass, target);

// DOC: Frame.constructor (begin)
const frame = new Frame();
// DOC: (end)
// DOC: Frame.add_pass (begin)
frame.addPass(rpass);
// DOC: (end)
renderer.render(frame, target);

// Additional API coverage for docs
// DOC: Shader.get (begin)
const _res = shader.get("resolution");
// DOC: (end)
// DOC: Shader.list_uniforms (begin)
const _uniforms = shader.listUniforms();
// DOC: (end)
// DOC: Shader.list_keys (begin)
const _keys = shader.listKeys();
// DOC: (end)

// Auto-generated: helpers in global scope
import { exampleShader } from './helpers.mjs';
Object.assign(globalThis, { exampleShader });

// Auto-generated: run all extracted examples (after init), unless in headless CI mode
const params = new URLSearchParams(globalThis.location?.search || '');
const isHeadless = params.get('mode') === 'headless' || params.has('headless');
if (isHeadless) {
  console.log('Headless JS render completed successfully');
} else {
  await import('./generated_examples.mjs');
}
