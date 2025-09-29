import { Renderer, Pass, Shader } from "fragmentcolor";

const renderer = new Renderer();
const target = await renderer.createTextureTarget([64u32, 64u32]);

// Create a depth texture usable as a per-pass attachment;
const depth = await renderer.createDepthTexture([64u32, 64u32]);

// Simple scene shader with @location(0) position;
const wgsl = r#";
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex;
fn vs_main(@location(0) pos: vec3<f32>) -> VOut { var o: VOut; o.pos = vec4f(pos,1.0); return o; };
@fragment;
fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4f(0.7,0.8,1.0,1.0); };
"#;
const shader = new Shader(wgsl);
const pass = new Pass("scene"); pass.addShader(shader);

// Attach depth texture to enable depth testing;
pass.addDepthTarget(depth);

// Render as usual;
renderer.render(pass, target);