// easing/in_out_expo — symmetric expo.
fn in_out_expo(t: f32) -> f32 {
  if (t <= 0.0) { return 0.0; }
  if (t >= 1.0) { return 1.0; }
  return select((2.0 - pow(2.0, -20.0 * t + 10.0)) * 0.5, pow(2.0, 20.0 * t - 10.0) * 0.5, t < 0.5);
}
