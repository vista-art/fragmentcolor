// sdf/cone — infinite cone with angle `c` (vec2 of sin, cos). Apex at origin, +y axis.
fn cone(p: vec3<f32>, c: vec2<f32>) -> f32 {
  let q = length(p.xz);
  return max(dot(c, vec2<f32>(q, p.y)), -p.y);
}
