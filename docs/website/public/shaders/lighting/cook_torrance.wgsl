// lighting/cook_torrance — specular BRDF = D * G * F / (4 * NdV * NdL).
// Inlines GGX D, Smith G, Schlick F. All vectors normalized internally.
fn _ct_ggx(ndh: f32, r: f32) -> f32 {
  let a = r * r; let a2 = a * a;
  let d = (ndh * ndh) * (a2 - 1.0) + 1.0;
  return a2 / (3.14159265 * d * d);
}

fn _ct_g1(cosv: f32, k: f32) -> f32 { return cosv / (cosv * (1.0 - k) + k); }

fn _ct_fschlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
  return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta, 5.0);
}

fn cook_torrance(n: vec3<f32>, l: vec3<f32>, v: vec3<f32>, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
  let nn = normalize(n); let ll = normalize(l); let vv = normalize(v);
  let h = normalize(ll + vv);
  let ndh = max(dot(nn, h), 0.0);
  let ndl = max(dot(nn, ll), 0.0);
  let ndv = max(dot(nn, vv), 0.0);
  let vdh = max(dot(vv, h), 0.0);
  let r1 = roughness + 1.0;
  let k = (r1 * r1) * 0.125;
  let D = _ct_ggx(ndh, roughness);
  let G = _ct_g1(ndv, k) * _ct_g1(ndl, k);
  let F = _ct_fschlick(vdh, f0);
  return (D * G * F) / max(4.0 * ndl * ndv, 1.0e-6);
}
