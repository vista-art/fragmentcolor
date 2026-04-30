// sdf2d/heart — heart shape centered near origin, scale `s`.
fn heart(p: vec2<f32>, s: f32) -> f32 {
  var q = vec2<f32>(abs(p.x), p.y) / s;
  q.y = q.y + 0.3;
  let a = length(q - vec2<f32>(0.25, 0.75)) - 0.25 * sqrt(2.0);
  let b = q - vec2<f32>(0.0, 1.0);
  let c = length(vec2<f32>(b.x, max(b.y + q.x * 0.0, 0.0)));
  let d = max(dot(q, vec2<f32>(0.7071, 0.7071)), 0.0) - 1.0;
  return s * min(a, d);
}
