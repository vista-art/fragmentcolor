import { Shader } from 'fragmentcolor';

export function exampleShader() {
  return new Shader(`
struct VertexOutput {
  @builtin(position) coords: vec4<f32>,
}
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
  const vertices = array(vec2(-1.,-1.), vec2(3.,-1.), vec2(-1.,3.));
  return VertexOutput(vec4<f32>(vertices[in_vertex_index], 0.0, 1.0));
}
@fragment
fn main() -> @location(0) vec4<f32> {
  return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
`);
}

