// material/pbr — physically-based shading function (Cook-Torrance specular
// with GGX D, Smith G, and Schlick F; Lambertian diffuse with energy
// conservation against the specular Fresnel).
//
// Self-contained: declares no uniforms, no bind groups. Pass everything in
// as args. Operates on a single light at a time — call once per light and
// sum the results, then add ambient or emissive separately.
//
// All inputs are normalized internally; pass raw world-space vectors.
//   n           : surface normal toward outside
//   l           : direction FROM surface TO the light
//   v           : direction FROM surface TO the camera
//   base_color  : albedo for dielectrics, F0 reflectance for metals (linear)
//   metallic    : 0 = pure dielectric, 1 = pure metal
//   roughness   : 0 = mirror, 1 = fully diffuse
//   light_color : light radiance (linear, already attenuated by your fall-off)

fn pbr_shade(
  n: vec3<f32>,
  l: vec3<f32>,
  v: vec3<f32>,
  base_color: vec3<f32>,
  metallic: f32,
  roughness: f32,
  light_color: vec3<f32>,
) -> vec3<f32> {
  let nn = normalize(n);
  let ll = normalize(l);
  let vv = normalize(v);
  let ndl = max(dot(nn, ll), 0.0);
  if (ndl <= 0.0) { return vec3<f32>(0.0); }

  let h = normalize(ll + vv);
  let ndh = max(dot(nn, h), 0.0);
  let ndv = max(dot(nn, vv), 0.0);
  let vdh = max(dot(vv, h), 0.0);

  // F0: 0.04 for dielectrics (water/plastic/etc.), base_color for metals.
  let f0 = mix(vec3<f32>(0.04), base_color, metallic);

  // GGX normal distribution.
  let a = roughness * roughness;
  let a2 = a * a;
  let d_denom = (ndh * ndh) * (a2 - 1.0) + 1.0;
  let D = a2 / (3.14159265 * d_denom * d_denom);

  // Smith geometry (Schlick-GGX form, direct-light remapping).
  let r1 = roughness + 1.0;
  let k = (r1 * r1) * 0.125;
  let G = (ndv / (ndv * (1.0 - k) + k)) * (ndl / (ndl * (1.0 - k) + k));

  // Schlick Fresnel.
  let F = f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - vdh, 5.0);

  let specular = (D * G * F) / max(4.0 * ndl * ndv, 1.0e-6);

  // Diffuse: only dielectrics scatter; metals absorb the refracted component.
  let kd = (vec3<f32>(1.0) - F) * (1.0 - metallic);
  let diffuse = kd * base_color / 3.14159265;

  return (diffuse + specular) * light_color * ndl;
}
