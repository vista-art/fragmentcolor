// sdf2d/ring — annulus with outer radius `r` and thickness `th`.
fn ring(p: vec2<f32>, r: f32, th: f32) -> f32 {
  return abs(length(p) - r) - th;
}
