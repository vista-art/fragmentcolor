// gradient/grayscale — linear gray ramp.
fn grayscale(t: f32) -> vec3<f32> {
  let x = clamp(t, 0.0, 1.0);
  return vec3<f32>(x);
}
