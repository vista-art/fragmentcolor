// sdf2d/trapezoid — isoceles trapezoid with top half-width r1, bottom r2, half-height h.
fn trapezoid(p: vec2<f32>, r1: f32, r2: f32, h: f32) -> f32 {
  let k1 = vec2<f32>(r2, h);
  let k2 = vec2<f32>(r2 - r1, 2.0 * h);
  var q = vec2<f32>(abs(p.x), p.y);
  let ca = vec2<f32>(q.x - min(q.x, select(r2, r1, q.y < 0.0)), abs(q.y) - h);
  let cb = q - k1 + k2 * clamp(dot(k1 - q, k2) / dot(k2, k2), 0.0, 1.0);
  let s = select(1.0, -1.0, cb.x < 0.0 && ca.y < 0.0);
  return s * sqrt(min(dot(ca, ca), dot(cb, cb)));
}
