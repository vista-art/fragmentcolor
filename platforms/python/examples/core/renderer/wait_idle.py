from fragmentcolor import Renderer, Shader

renderer = Renderer()
target = renderer.create_texture_target((8, 8))
shader = Shader("""
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4f(1., 1., 1., 1.); }
""")
renderer.render(shader, target)
renderer.wait_idle()
bytes = target.get_image()
