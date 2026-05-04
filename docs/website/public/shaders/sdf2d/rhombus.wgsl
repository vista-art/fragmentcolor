// sdf2d/rhombus — rhombus with axis half-lengths `b` (x = horizontal, y = vertical).
fn _rh_ndot(a: vec2<f32>, b: vec2<f32>) -> f32 { return a.x * b.x - a.y * b.y; }

fn rhombus(p: vec2<f32>, b: vec2<f32>) -> f32 {
  let q = abs(p);
  let h = clamp(_rh_ndot(b - 2.0 * q, b) / dot(b, b), -1.0, 1.0);
  let d = length(q - 0.5 * b * vec2<f32>(1.0 - h, 1.0 + h));
  return d * sign(q.x * b.y + q.y * b.x - b.x * b.y);
}
