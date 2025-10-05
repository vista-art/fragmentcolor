
import { Renderer, Shader } from "fragmentcolor";
const renderer = new Renderer();
const shader = new Shader(`
@group(0) @binding(0) var my_texture: texture_2d<f32>;
@group(0) @binding(1) var my_sampler: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }

`);

// 1x1 RGBA (white) raw pixel bytes
const pixels = [255,255,255,255];
const texture = await renderer.createTextureWithSize(pixels, [1,1]);

// insert  the texture in the shader matching the name in the shader
shader.set("my_texture", texture);
