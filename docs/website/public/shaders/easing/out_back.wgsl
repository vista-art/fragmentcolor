// easing/out_back — overshoot past end then settle.
fn out_back(t: f32) -> f32 {
  let c1 = 1.70158;
  let c3 = c1 + 1.0;
  let u = t - 1.0;
  return 1.0 + c3 * u * u * u + c1 * u * u;
}
