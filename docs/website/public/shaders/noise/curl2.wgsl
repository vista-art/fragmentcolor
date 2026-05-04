// curl2 — 2D curl noise, i.e. divergence-free 2D vector field derived from
// a scalar noise potential by 90° rotation of the gradient. Good for fluid-like motion.
fn _curl2_hash(p: vec2<f32>) -> f32 {
  var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

fn _curl2_value(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(_curl2_hash(i), _curl2_hash(i + vec2<f32>(1.0, 0.0)), u.x),
    mix(_curl2_hash(i + vec2<f32>(0.0, 1.0)), _curl2_hash(i + vec2<f32>(1.0, 1.0)), u.x),
    u.y
  );
}

fn curl2(p: vec2<f32>) -> vec2<f32> {
  let e = 0.01;
  let n_x1 = _curl2_value(p + vec2<f32>(e, 0.0));
  let n_x0 = _curl2_value(p - vec2<f32>(e, 0.0));
  let n_y1 = _curl2_value(p + vec2<f32>(0.0, e));
  let n_y0 = _curl2_value(p - vec2<f32>(0.0, e));
  let dn_dx = (n_x1 - n_x0) / (2.0 * e);
  let dn_dy = (n_y1 - n_y0) / (2.0 * e);
  return vec2<f32>(dn_dy, -dn_dx);
}
