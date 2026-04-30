// easing/out_cubic — 1 - (1 - t)^3.
fn out_cubic(t: f32) -> f32 {
  let u = 1.0 - t;
  return 1.0 - u * u * u;
}
