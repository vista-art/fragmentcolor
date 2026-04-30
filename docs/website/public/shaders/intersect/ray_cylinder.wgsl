// intersect/ray_cylinder — ray vs infinite cylinder with axis along +Y, centered at origin,
// radius `r`. Returns (t_near, t_far) or (-1, -1) on miss.
fn ray_cylinder(ro: vec3<f32>, rd: vec3<f32>, r: f32) -> vec2<f32> {
  let rxz = vec2<f32>(rd.x, rd.z);
  let oxz = vec2<f32>(ro.x, ro.z);
  let a = dot(rxz, rxz);
  let b = dot(oxz, rxz);
  let c = dot(oxz, oxz) - r * r;
  let h = b * b - a * c;
  if (h < 0.0 || a < 1.0e-8) { return vec2<f32>(-1.0); }
  let sq = sqrt(h);
  return vec2<f32>((-b - sq) / a, (-b + sq) / a);
}
