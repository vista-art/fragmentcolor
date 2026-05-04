// postfx/invert — 1 - color, channel-wise.
fn invert(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(1.0) - c;
}
