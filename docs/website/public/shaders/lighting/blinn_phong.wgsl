// lighting/blinn_phong — Blinn half-vector specular. Usually preferred over Phong.
fn blinn_phong(n: vec3<f32>, l: vec3<f32>, v: vec3<f32>, shine: f32) -> f32 {
  let h = normalize(normalize(l) + normalize(v));
  return pow(max(dot(normalize(n), h), 0.0), shine);
}
