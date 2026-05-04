// map/cartesian — inverse of polar: (angle/TAU, radius) → uv around center.
fn cartesian(pr: vec2<f32>, center: vec2<f32>) -> vec2<f32> {
  let a = pr.x * 6.28318530718;
  return center + pr.y * vec2<f32>(cos(a), sin(a));
}
