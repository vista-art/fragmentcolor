// saturate — clamps a value to [0, 1].
fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}
