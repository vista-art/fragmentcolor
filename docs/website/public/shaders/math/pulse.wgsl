// pulse — 1.0 inside [a, b], 0.0 outside. Hard edges.
fn pulse(a: f32, b: f32, x: f32) -> f32 {
  return step(a, x) - step(b, x);
}
