// pattern/truchet — Truchet tiles with random rotation per cell. Returns the
// minimum distance to a quarter-arc through two adjacent corners of each cell.
fn _tru_hash(p: vec2<f32>) -> f32 {
  var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

fn truchet(uv: vec2<f32>, scale: f32) -> f32 {
  let p = uv * scale;
  let i = floor(p);
  var f = fract(p);
  let r = _tru_hash(i);
  if (r < 0.5) { f = vec2<f32>(f.y, 1.0 - f.x); }
  let d1 = abs(length(f) - 0.5);
  let d2 = abs(length(f - vec2<f32>(1.0)) - 0.5);
  return min(d1, d2);
}
