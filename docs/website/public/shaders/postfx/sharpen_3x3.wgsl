// postfx/sharpen_3x3 — apply a center-weighted sharpen kernel to a 3x3 neighborhood.
// Pass the 9 already-sampled colors in row-major order (c00 top-left ... c22 bottom-right).
// `amount` 0 returns center color, 1 returns strong sharpen.
fn sharpen_3x3(
  c00: vec3<f32>, c10: vec3<f32>, c20: vec3<f32>,
  c01: vec3<f32>, c11: vec3<f32>, c21: vec3<f32>,
  c02: vec3<f32>, c12: vec3<f32>, c22: vec3<f32>,
  amount: f32
) -> vec3<f32> {
  let sum_neighbors = c10 + c01 + c21 + c12;
  let sharpened = c11 * (1.0 + 4.0 * amount) - sum_neighbors * amount;
  return sharpened;
}
