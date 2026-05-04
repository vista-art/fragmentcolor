// lighting/oren_nayar — rough-surface diffuse; reduces retroreflective bleed.
fn oren_nayar(n: vec3<f32>, l: vec3<f32>, v: vec3<f32>, roughness: f32) -> f32 {
  let nn = normalize(n); let ll = normalize(l); let vv = normalize(v);
  let ndl = max(dot(nn, ll), 0.0);
  let ndv = max(dot(nn, vv), 0.0);
  let r2 = roughness * roughness;
  let A = 1.0 - 0.5 * r2 / (r2 + 0.33);
  let B = 0.45 * r2 / (r2 + 0.09);
  let gamma = dot(vv - nn * ndv, ll - nn * ndl);
  let alpha = max(acos(ndv), acos(ndl));
  let beta  = min(acos(ndv), acos(ndl));
  return ndl * (A + B * max(0.0, gamma) * sin(alpha) * tan(beta));
}
