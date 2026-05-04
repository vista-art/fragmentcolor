// lighting/phong — classic Phong specular term. `shine` is specular exponent.
fn phong(n: vec3<f32>, l: vec3<f32>, v: vec3<f32>, shine: f32) -> f32 {
  let r = reflect(-normalize(l), normalize(n));
  return pow(max(dot(r, normalize(v)), 0.0), shine);
}
