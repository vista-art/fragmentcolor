// geom/rotate_3d_z — 3x3 rotation around +Z by `a` radians.
fn rotate_3d_z(a: f32) -> mat3x3<f32> {
  let s = sin(a); let c = cos(a);
  return mat3x3<f32>(
    vec3<f32>(  c,   s, 0.0),
    vec3<f32>( -s,   c, 0.0),
    vec3<f32>(0.0, 0.0, 1.0)
  );
}
