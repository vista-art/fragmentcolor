// ridged2 — 2D ridged multifractal. Produces crisp mountain-ridge patterns.
fn _ridged2_hash(p: vec2<f32>) -> f32 {
  var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

fn _ridged2_value(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(_ridged2_hash(i), _ridged2_hash(i + vec2<f32>(1.0, 0.0)), u.x),
    mix(_ridged2_hash(i + vec2<f32>(0.0, 1.0)), _ridged2_hash(i + vec2<f32>(1.0, 1.0)), u.x),
    u.y
  );
}

fn ridged2(p: vec2<f32>, octaves: u32) -> f32 {
  var v = 0.0;
  var a = 0.5;
  var q = p;
  for (var i = 0u; i < octaves; i = i + 1u) {
    let n = abs(_ridged2_value(q) * 2.0 - 1.0);
    v = v + a * (1.0 - n);
    q = q * 2.03;
    a = a * 0.5;
  }
  return v;
}
