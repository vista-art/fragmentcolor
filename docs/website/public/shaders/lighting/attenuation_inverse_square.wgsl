// lighting/attenuation_inverse_square — physically-based 1 / d² with min-distance guard.
fn attenuation_inverse_square(distance: f32, min_d: f32) -> f32 {
  let d = max(distance, min_d);
  return 1.0 / (d * d);
}
