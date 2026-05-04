// saturation — adjust saturation by mixing toward Rec.709 luminance.
// s < 1 desaturates, s > 1 amplifies, s == 0 grayscale.
fn saturation(c: vec3<f32>, s: f32) -> vec3<f32> {
  let l = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
  return mix(vec3<f32>(l), c, s);
}
