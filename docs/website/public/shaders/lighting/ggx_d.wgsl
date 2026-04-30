// lighting/ggx_d — GGX / Trowbridge-Reitz normal distribution function.
fn ggx_d(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
  let a  = roughness * roughness;
  let a2 = a * a;
  let ndh = max(dot(normalize(n), normalize(h)), 0.0);
  let d  = (ndh * ndh) * (a2 - 1.0) + 1.0;
  return a2 / (3.14159265 * d * d);
}
