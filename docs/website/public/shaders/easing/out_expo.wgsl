// easing/out_expo — 1 - 2^(-10 t) with snap at t == 1.
fn out_expo(t: f32) -> f32 {
  return select(1.0 - pow(2.0, -10.0 * t), 1.0, t >= 1.0);
}
