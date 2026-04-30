// sdf/op_smooth_subtract — smooth subtraction (a - b) with blend `k`.
fn op_smooth_subtract(a: f32, b: f32, k: f32) -> f32 {
  let h = clamp(0.5 - 0.5 * (b + a) / k, 0.0, 1.0);
  return mix(a, -b, h) + k * h * (1.0 - h);
}
