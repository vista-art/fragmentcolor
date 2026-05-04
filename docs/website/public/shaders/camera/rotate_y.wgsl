// camera/rotate_y — 4x4 rotation around the Y axis by `a` radians.
fn rotate_y(a: f32) -> mat4x4<f32> {
  let s = sin(a); let c = cos(a);
  return mat4x4<f32>(
    vec4<f32>(  c, 0.0,  -s, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(  s, 0.0,   c, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
  );
}
