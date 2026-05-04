// geom/triangle_normal — unit face normal from triangle vertices.
fn triangle_normal(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
  return normalize(cross(b - a, c - a));
}
