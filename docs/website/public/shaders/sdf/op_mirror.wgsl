// sdf/op_mirror — reflect negative half of space onto positive along axis `a` (must be unit).
fn op_mirror(p: vec3<f32>, a: vec3<f32>) -> vec3<f32> {
  return p - 2.0 * min(dot(p, a), 0.0) * a;
}
