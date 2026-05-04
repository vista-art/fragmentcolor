// intersect/ray_box — ray vs AABB with half-extents `b` centered at `center`.
// Returns (t_near, t_far) or (-1, -1) on miss. Slab method.
fn ray_box(ro: vec3<f32>, rd: vec3<f32>, center: vec3<f32>, b: vec3<f32>) -> vec2<f32> {
  let inv = 1.0 / rd;
  let mn = (center - b - ro) * inv;
  let mx = (center + b - ro) * inv;
  let t1 = min(mn, mx);
  let t2 = max(mn, mx);
  let t_near = max(max(t1.x, t1.y), t1.z);
  let t_far  = min(min(t2.x, t2.y), t2.z);
  if (t_far < max(t_near, 0.0)) { return vec2<f32>(-1.0); }
  return vec2<f32>(t_near, t_far);
}
