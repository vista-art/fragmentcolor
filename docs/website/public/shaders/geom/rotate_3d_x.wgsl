// geom/rotate_3d_x — 3x3 rotation around +X by `a` radians.
fn rotate_3d_x(a: f32) -> mat3x3<f32> {
  let s = sin(a); let c = cos(a);
  return mat3x3<f32>(
    vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0,   c,   s),
    vec3<f32>(0.0,  -s,   c)
  );
}
