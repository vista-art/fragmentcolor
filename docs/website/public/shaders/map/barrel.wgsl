// map/barrel — radial lens distortion around UV center (0.5, 0.5).
// k > 0 barrel (bulge out), k < 0 pincushion.
fn barrel(uv: vec2<f32>, k: f32) -> vec2<f32> {
  let c = uv - vec2<f32>(0.5);
  let r2 = dot(c, c);
  return vec2<f32>(0.5) + c * (1.0 + k * r2);
}
