// fbm3 — 3D fractal Brownian motion. Output in [0, 1].
fn _fbm3_hash(p: vec3<f32>) -> f32 {
  var p3 = fract(p * 0.1031);
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

fn _fbm3_value(p: vec3<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  let c000 = _fbm3_hash(i + vec3<f32>(0.0, 0.0, 0.0));
  let c100 = _fbm3_hash(i + vec3<f32>(1.0, 0.0, 0.0));
  let c010 = _fbm3_hash(i + vec3<f32>(0.0, 1.0, 0.0));
  let c110 = _fbm3_hash(i + vec3<f32>(1.0, 1.0, 0.0));
  let c001 = _fbm3_hash(i + vec3<f32>(0.0, 0.0, 1.0));
  let c101 = _fbm3_hash(i + vec3<f32>(1.0, 0.0, 1.0));
  let c011 = _fbm3_hash(i + vec3<f32>(0.0, 1.0, 1.0));
  let c111 = _fbm3_hash(i + vec3<f32>(1.0, 1.0, 1.0));
  let x00 = mix(c000, c100, u.x);
  let x10 = mix(c010, c110, u.x);
  let x01 = mix(c001, c101, u.x);
  let x11 = mix(c011, c111, u.x);
  return mix(mix(x00, x10, u.y), mix(x01, x11, u.y), u.z);
}

fn fbm3(p: vec3<f32>, octaves: u32) -> f32 {
  var v = 0.0;
  var a = 0.5;
  var q = p;
  for (var i = 0u; i < octaves; i = i + 1u) {
    v = v + a * _fbm3_value(q);
    q = q * 2.03;
    a = a * 0.5;
  }
  return v;
}
