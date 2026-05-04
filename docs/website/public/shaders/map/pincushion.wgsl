// map/pincushion — inverse of barrel distortion (k always positive for pincushion).
fn pincushion(uv: vec2<f32>, k: f32) -> vec2<f32> {
  let c = uv - vec2<f32>(0.5);
  let r2 = dot(c, c);
  return vec2<f32>(0.5) + c / (1.0 + k * r2);
}
