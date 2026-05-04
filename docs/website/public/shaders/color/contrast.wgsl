// contrast — linear contrast around mid-gray (0.5). c: color, k: factor (1 = identity).
fn contrast(c: vec3<f32>, k: f32) -> vec3<f32> {
  return (c - vec3<f32>(0.5)) * k + vec3<f32>(0.5);
}
