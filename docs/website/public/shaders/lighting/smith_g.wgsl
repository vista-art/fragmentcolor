// lighting/smith_g — Smith's masking+shadowing geometry term (Schlick-GGX G1 combined).
fn _smith_g1(ndv: f32, k: f32) -> f32 {
  return ndv / (ndv * (1.0 - k) + k);
}

fn smith_g(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
  let r = roughness + 1.0;
  let k = (r * r) * 0.125;
  let ndv = max(dot(normalize(n), normalize(v)), 0.0);
  let ndl = max(dot(normalize(n), normalize(l)), 0.0);
  return _smith_g1(ndv, k) * _smith_g1(ndl, k);
}
