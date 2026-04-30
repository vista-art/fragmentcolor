// easing/out_quart — 1 - (1 - t)^4.
fn out_quart(t: f32) -> f32 {
  let u = 1.0 - t;
  return 1.0 - u * u * u * u;
}
