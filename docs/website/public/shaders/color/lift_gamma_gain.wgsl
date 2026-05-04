// lift_gamma_gain — DaVinci-style color grade. lift/gain scale shadows/highlights;
// gamma shapes midtones. Typical ranges: lift [-0.5, 0.5], gamma [0.1, 3.0], gain [0.5, 2.0].
fn lift_gamma_gain(c: vec3<f32>, lift: vec3<f32>, gamma: vec3<f32>, gain: vec3<f32>) -> vec3<f32> {
  let lifted = c + lift * (vec3<f32>(1.0) - c);
  let gained = lifted * gain;
  return pow(max(gained, vec3<f32>(0.0)), vec3<f32>(1.0) / gamma);
}
