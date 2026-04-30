// intersect/ray_plane — ray vs infinite plane with unit normal `n` at origin-distance `d`.
// Returns t (may be negative); returns a large sentinel 1e30 on miss (parallel ray).
fn ray_plane(ro: vec3<f32>, rd: vec3<f32>, n: vec3<f32>, d: f32) -> f32 {
  let denom = dot(rd, n);
  if (abs(denom) < 1.0e-6) { return 1.0e30; }
  return -(dot(ro, n) + d) / denom;
}
