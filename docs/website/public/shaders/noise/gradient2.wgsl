// gradient2 — 2D gradient (Perlin) noise in roughly [-1, 1].
fn _gn2_hash(p: vec2<f32>) -> vec2<f32> {
  var p3 = fract(vec3<f32>(p.x, p.y, p.x) * vec3<f32>(0.1031, 0.1030, 0.0973));
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.xx + p3.yz) * p3.zy) * 2.0 - 1.0;
}

fn gradient2(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  let g00 = _gn2_hash(i + vec2<f32>(0.0, 0.0));
  let g10 = _gn2_hash(i + vec2<f32>(1.0, 0.0));
  let g01 = _gn2_hash(i + vec2<f32>(0.0, 1.0));
  let g11 = _gn2_hash(i + vec2<f32>(1.0, 1.0));
  let n00 = dot(g00, f - vec2<f32>(0.0, 0.0));
  let n10 = dot(g10, f - vec2<f32>(1.0, 0.0));
  let n01 = dot(g01, f - vec2<f32>(0.0, 1.0));
  let n11 = dot(g11, f - vec2<f32>(1.0, 1.0));
  return mix(mix(n00, n10, u.x), mix(n01, n11, u.x), u.y);
}
