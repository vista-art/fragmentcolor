// lighting/half_lambert — Valve-style wrapped diffuse, ((n·l)*0.5 + 0.5)^2. Softer shadows.
fn half_lambert(n: vec3<f32>, l: vec3<f32>) -> f32 {
  let ndl = dot(normalize(n), normalize(l)) * 0.5 + 0.5;
  return ndl * ndl;
}
