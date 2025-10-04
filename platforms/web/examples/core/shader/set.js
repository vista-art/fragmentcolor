import { Renderer, Shader } from "fragmentcolor";
const r = new Renderer();
const shader = new Shader(`
@group(0) @binding(0) var<uniform> resolution: vec2<f32>;

struct VOut { @builtin(position) pos: vec4<f32> };
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
  var out: VOut;
  out.pos = vec4<f32>(p[i], 0., 1.);
  return out;
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }

`);

// Set scalars/vectors on declared uniforms
shader.set("resolution", [800.0, 600.0]);