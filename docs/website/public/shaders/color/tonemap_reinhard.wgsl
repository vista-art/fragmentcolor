// tonemap_reinhard — Reinhard: c / (1 + c). Cheap but washes highlights.
fn tonemap_reinhard(c: vec3<f32>) -> vec3<f32> {
  return c / (vec3<f32>(1.0) + c);
}
