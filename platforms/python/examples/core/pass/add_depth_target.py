from fragmentcolor import Renderer, Pass, Shader

renderer = Renderer()
target = renderer.create_texture_target([64u32, 64u32])

# Create a depth texture usable as a per-pass attachment
depth = renderer.create_depth_texture([64u32, 64u32])

# Simple scene shader with @location(0) position
wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> }
@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VOut { var o: VOut; o.pos = vec4f(pos,1.0); return o; }
@fragment
fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4f(0.7,0.8,1.0,1.0); }
"#
shader = Shader(wgsl)
rpass = Pass("scene"); rpass.add_shader(shader)

# Attach depth texture to enable depth testing
rpass.add_depth_target(depth)

# Render as usual
renderer.render(rpass, target)