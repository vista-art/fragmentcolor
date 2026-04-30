// easing/out_elastic — elastic settle to 1.
fn out_elastic(t: f32) -> f32 {
  let c4 = 6.28318530718 / 3.0;
  if (t <= 0.0) { return 0.0; }
  if (t >= 1.0) { return 1.0; }
  return pow(2.0, -10.0 * t) * sin((t * 10.0 - 0.75) * c4) + 1.0;
}
