// lighting/disney_diffuse — Burley's Disney diffuse term (energy-preserving, rougher edges).
fn disney_diffuse(n: vec3<f32>, l: vec3<f32>, v: vec3<f32>, roughness: f32) -> f32 {
  let nn = normalize(n); let ll = normalize(l); let vv = normalize(v);
  let h = normalize(ll + vv);
  let ndl = max(dot(nn, ll), 0.0);
  let ndv = max(dot(nn, vv), 0.0);
  let ldh = max(dot(ll, h), 0.0);
  let fd90 = 0.5 + 2.0 * ldh * ldh * roughness;
  let f_l = 1.0 + (fd90 - 1.0) * pow(1.0 - ndl, 5.0);
  let f_v = 1.0 + (fd90 - 1.0) * pow(1.0 - ndv, 5.0);
  return ndl * f_l * f_v / 3.14159265;
}
