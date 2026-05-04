// value2 — 2D value noise in [0, 1). Inlines hash21-style helper.
fn _vn2_hash(p: vec2<f32>) -> f32 {
  var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

fn value2(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(_vn2_hash(i + vec2<f32>(0.0, 0.0)), _vn2_hash(i + vec2<f32>(1.0, 0.0)), u.x),
    mix(_vn2_hash(i + vec2<f32>(0.0, 1.0)), _vn2_hash(i + vec2<f32>(1.0, 1.0)), u.x),
    u.y
  );
}
