// sdf2d/circle — signed distance to a circle of radius `r` at the origin.
fn circle(p: vec2<f32>, r: f32) -> f32 {
  return length(p) - r;
}
