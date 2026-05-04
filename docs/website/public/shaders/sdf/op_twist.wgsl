// sdf/op_twist — domain twist around the Y axis. Returns transformed point.
fn op_twist(p: vec3<f32>, k: f32) -> vec3<f32> {
  let c = cos(k * p.y);
  let s = sin(k * p.y);
  let m = mat2x2<f32>(vec2<f32>(c, s), vec2<f32>(-s, c));
  let xz = m * vec2<f32>(p.x, p.z);
  return vec3<f32>(xz.x, p.y, xz.y);
}
