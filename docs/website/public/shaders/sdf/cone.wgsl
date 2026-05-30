// sdf/cone — infinite cone with angle `c` (vec2 of sin, cos). Apex at origin,
// axis along y, opening toward -y. Cap it to a finite cone at the call site
// with `max(cone(p, c), -h - p.y)` for a base `h` below the apex.
fn cone(p: vec3<f32>, c: vec2<f32>) -> f32 {
  let q = length(p.xz);
  return dot(c, vec2<f32>(q, p.y));
}
