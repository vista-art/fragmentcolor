// lighting/fresnel_schlick_roughness — Sébastien Lagarde's roughness-aware Schlick,
// used for IBL with pre-integrated env maps.
fn fresnel_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
  return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * pow(1.0 - cos_theta, 5.0);
}
