// lighting/attenuation_range — inverse-square falloff clamped to a finite `range` (Unreal-style).
fn attenuation_range(distance: f32, range: f32) -> f32 {
  let d2 = distance * distance;
  let r2 = range * range;
  let n = clamp(1.0 - (d2 * d2) / (r2 * r2), 0.0, 1.0);
  return (n * n) / (d2 + 1.0);
}
