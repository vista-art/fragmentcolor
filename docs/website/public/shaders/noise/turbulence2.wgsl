// turbulence2 — 2D turbulence (fbm over |noise|). Gives flame/smoke feel.
fn _turb2_hash(p: vec2<f32>) -> f32 {
  var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

fn _turb2_value(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(_turb2_hash(i), _turb2_hash(i + vec2<f32>(1.0, 0.0)), u.x),
    mix(_turb2_hash(i + vec2<f32>(0.0, 1.0)), _turb2_hash(i + vec2<f32>(1.0, 1.0)), u.x),
    u.y
  );
}

fn turbulence2(p: vec2<f32>, octaves: u32) -> f32 {
  var v = 0.0;
  var a = 0.5;
  var q = p;
  for (var i = 0u; i < octaves; i = i + 1u) {
    v = v + a * abs(_turb2_value(q) * 2.0 - 1.0);
    q = q * 2.03;
    a = a * 0.5;
  }
  return v;
}
