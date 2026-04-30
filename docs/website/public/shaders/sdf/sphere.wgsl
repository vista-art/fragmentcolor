// sdf/sphere — signed distance to a sphere centered at the origin.
fn sphere(p: vec3<f32>, r: f32) -> f32 {
  return length(p) - r;
}
