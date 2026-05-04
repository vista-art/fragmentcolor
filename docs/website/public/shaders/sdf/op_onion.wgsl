// sdf/op_onion — hollow shell: takes a signed distance and returns a thin shell of thickness `t`.
fn op_onion(d: f32, t: f32) -> f32 {
  return abs(d) - t;
}
