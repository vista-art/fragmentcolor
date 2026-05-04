// oklch_to_linear_srgb — OkLCh (polar OkLab: L, C, h with h in radians) → linear sRGB.
fn oklch_to_linear_srgb(lch: vec3<f32>) -> vec3<f32> {
  let l = lch.x;
  let a = lch.y * cos(lch.z);
  let b = lch.y * sin(lch.z);
  let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
  let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
  let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
  let L = l_ * l_ * l_;
  let M = m_ * m_ * m_;
  let S = s_ * s_ * s_;
  return vec3<f32>(
     4.0767416621 * L - 3.3077115913 * M + 0.2309699292 * S,
    -1.2684380046 * L + 2.6097574011 * M - 0.3413193965 * S,
    -0.0041960863 * L - 0.7034186147 * M + 1.7076147010 * S
  );
}
