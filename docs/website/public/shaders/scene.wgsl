// FragmentColor demo shader: scene
// Produces a soft color field using UV coordinates.

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VSOut {
  var out: VSOut;
  var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 3.0, -1.0),
    vec2<f32>(-1.0,  3.0),
  );
  let p = positions[vi];
  out.pos = vec4<f32>(p, 0.0, 1.0);
  out.uv = p * 0.5 + vec2<f32>(0.5, 0.5);
  return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
  let uv = in.uv;
  // Simple color field with gentle bands
  let r = 0.5 + 0.5 * sin(6.2831 * (uv.x * 0.5 + uv.y * 0.2));
  let g = 0.4 + 0.6 * smoothstep(0.0, 1.0, uv.y);
  let b = 0.6 + 0.4 * cos(6.2831 * (uv.x * 0.3 - uv.y * 0.1));
  return vec4<f32>(r, g, b, 1.0);
}
