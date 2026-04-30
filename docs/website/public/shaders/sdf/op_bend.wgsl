// sdf/op_bend — domain bend around the Z axis. Returns transformed point.
fn op_bend(p: vec3<f32>, k: f32) -> vec3<f32> {
  let c = cos(k * p.x);
  let s = sin(k * p.x);
  let m = mat2x2<f32>(vec2<f32>(c, s), vec2<f32>(-s, c));
  let xy = m * vec2<f32>(p.x, p.y);
  return vec3<f32>(xy.x, xy.y, p.z);
}
