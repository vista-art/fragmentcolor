// easing/in_expo — 2^(10*(t-1)) with snap at t == 0.
fn in_expo(t: f32) -> f32 {
  return select(pow(2.0, 10.0 * (t - 1.0)), 0.0, t <= 0.0);
}
