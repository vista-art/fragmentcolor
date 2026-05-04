// sdf/op_repeat — infinite grid repetition of a point p with cell size c.
fn op_repeat(p: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
  return p - c * round(p / c);
}
