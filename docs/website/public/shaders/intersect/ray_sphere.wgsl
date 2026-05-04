// intersect/ray_sphere — ray vs sphere. Returns (t_near, t_far) or (-1, -1) on miss.
// Ray: ro + rd * t, rd assumed unit length.
fn ray_sphere(ro: vec3<f32>, rd: vec3<f32>, center: vec3<f32>, radius: f32) -> vec2<f32> {
  let oc = ro - center;
  let b = dot(oc, rd);
  let c = dot(oc, oc) - radius * radius;
  let h = b * b - c;
  if (h < 0.0) { return vec2<f32>(-1.0); }
  let sq = sqrt(h);
  return vec2<f32>(-b - sq, -b + sq);
}
