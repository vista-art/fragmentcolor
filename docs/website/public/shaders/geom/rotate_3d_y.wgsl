// geom/rotate_3d_y — 3x3 rotation around +Y by `a` radians.
fn rotate_3d_y(a: f32) -> mat3x3<f32> {
  let s = sin(a); let c = cos(a);
  return mat3x3<f32>(
    vec3<f32>(  c, 0.0,  -s),
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(  s, 0.0,   c)
  );
}
