// geom/triangle_area — 3D triangle area (half cross-product magnitude).
fn triangle_area(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> f32 {
  return 0.5 * length(cross(b - a, c - a));
}
