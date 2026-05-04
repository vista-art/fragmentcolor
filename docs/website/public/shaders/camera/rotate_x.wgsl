// camera/rotate_x — 4x4 rotation around the X axis by `a` radians.
fn rotate_x(a: f32) -> mat4x4<f32> {
  let s = sin(a); let c = cos(a);
  return mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0,   c,   s, 0.0),
    vec4<f32>(0.0,  -s,   c, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
  );
}
