// tonemap_filmic — Jim Hejl / Richard Burgess-Dawson curve (no sRGB encode needed).
fn tonemap_filmic(c: vec3<f32>) -> vec3<f32> {
  let x = max(vec3<f32>(0.0), c - vec3<f32>(0.004));
  return (x * (6.2 * x + vec3<f32>(0.5))) / (x * (6.2 * x + vec3<f32>(1.7)) + vec3<f32>(0.06));
}
