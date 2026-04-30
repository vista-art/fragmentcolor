// lighting/lambert — Lambertian diffuse term. Pass unnormalized `n` and `l`; normalized inside.
fn lambert(n: vec3<f32>, l: vec3<f32>) -> f32 {
  return max(dot(normalize(n), normalize(l)), 0.0);
}
