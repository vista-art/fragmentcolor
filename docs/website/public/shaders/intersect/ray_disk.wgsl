// intersect/ray_disk — ray vs finite disk at `center` with unit normal `n`, radius `r`.
// Returns t or -1 on miss.
fn ray_disk(ro: vec3<f32>, rd: vec3<f32>, center: vec3<f32>, n: vec3<f32>, r: f32) -> f32 {
  let denom = dot(rd, n);
  if (abs(denom) < 1.0e-6) { return -1.0; }
  let t = dot(center - ro, n) / denom;
  if (t <= 0.0) { return -1.0; }
  let p = ro + rd * t;
  if (length(p - center) > r) { return -1.0; }
  return t;
}
