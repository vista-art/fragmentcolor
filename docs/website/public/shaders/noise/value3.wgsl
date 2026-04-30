// value3 — 3D value noise in [0, 1). Inlines hash helper.
fn _vn3_hash(p: vec3<f32>) -> f32 {
  var p3 = fract(p * 0.1031);
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

fn value3(p: vec3<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  let c000 = _vn3_hash(i + vec3<f32>(0.0, 0.0, 0.0));
  let c100 = _vn3_hash(i + vec3<f32>(1.0, 0.0, 0.0));
  let c010 = _vn3_hash(i + vec3<f32>(0.0, 1.0, 0.0));
  let c110 = _vn3_hash(i + vec3<f32>(1.0, 1.0, 0.0));
  let c001 = _vn3_hash(i + vec3<f32>(0.0, 0.0, 1.0));
  let c101 = _vn3_hash(i + vec3<f32>(1.0, 0.0, 1.0));
  let c011 = _vn3_hash(i + vec3<f32>(0.0, 1.0, 1.0));
  let c111 = _vn3_hash(i + vec3<f32>(1.0, 1.0, 1.0));
  let x00 = mix(c000, c100, u.x);
  let x10 = mix(c010, c110, u.x);
  let x01 = mix(c001, c101, u.x);
  let x11 = mix(c011, c111, u.x);
  let y0 = mix(x00, x10, u.y);
  let y1 = mix(x01, x11, u.y);
  return mix(y0, y1, u.z);
}
