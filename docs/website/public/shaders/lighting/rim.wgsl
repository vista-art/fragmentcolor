// lighting/rim — rim / back-light term proportional to (1 - n·v)^power.
fn rim(n: vec3<f32>, v: vec3<f32>, power: f32) -> f32 {
  return pow(1.0 - max(dot(normalize(n), normalize(v)), 0.0), power);
}
