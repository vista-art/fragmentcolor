// postfx/threshold — binary step per channel at threshold `t`.
fn threshold(c: vec3<f32>, t: f32) -> vec3<f32> {
  return step(vec3<f32>(t), c);
}
