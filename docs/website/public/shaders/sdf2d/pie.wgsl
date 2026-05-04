// sdf2d/pie — pie-slice of angle `c` (sin, cos of half-angle) and radius `r`.
fn pie(p: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
  var q = vec2<f32>(abs(p.x), p.y);
  let l = length(q) - r;
  let m = length(q - c * clamp(dot(q, c), 0.0, r));
  return max(l, m * sign(c.y * q.x - c.x * q.y));
}
