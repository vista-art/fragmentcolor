// intersect/ray_triangle — Möller-Trumbore. Returns vec4(t, u, v, 1) on hit, (-1,-1,-1,-1) on miss.
fn ray_triangle(ro: vec3<f32>, rd: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec4<f32> {
  let e1 = b - a;
  let e2 = c - a;
  let p = cross(rd, e2);
  let det = dot(e1, p);
  if (abs(det) < 1.0e-8) { return vec4<f32>(-1.0); }
  let inv = 1.0 / det;
  let s = ro - a;
  let u = dot(s, p) * inv;
  if (u < 0.0 || u > 1.0) { return vec4<f32>(-1.0); }
  let q = cross(s, e1);
  let v = dot(rd, q) * inv;
  if (v < 0.0 || u + v > 1.0) { return vec4<f32>(-1.0); }
  let t = dot(e2, q) * inv;
  if (t <= 0.0) { return vec4<f32>(-1.0); }
  return vec4<f32>(t, u, v, 1.0);
}
