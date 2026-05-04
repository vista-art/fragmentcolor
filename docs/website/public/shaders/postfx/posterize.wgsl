// postfx/posterize — quantize a color to N steps per channel.
fn posterize(c: vec3<f32>, steps: f32) -> vec3<f32> {
  return floor(c * steps + vec3<f32>(0.5)) / steps;
}
