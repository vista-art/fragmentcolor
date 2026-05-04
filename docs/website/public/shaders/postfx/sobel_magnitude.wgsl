// postfx/sobel_magnitude — edge magnitude from a 3x3 luminance neighborhood.
// Pass the 9 luminances (row-major, l00 top-left ... l22 bottom-right).
fn sobel_magnitude(
  l00: f32, l10: f32, l20: f32,
  l01: f32, l11: f32, l21: f32,
  l02: f32, l12: f32, l22: f32
) -> f32 {
  let gx = (l20 + 2.0 * l21 + l22) - (l00 + 2.0 * l01 + l02);
  let gy = (l02 + 2.0 * l12 + l22) - (l00 + 2.0 * l10 + l20);
  return sqrt(gx * gx + gy * gy);
}
