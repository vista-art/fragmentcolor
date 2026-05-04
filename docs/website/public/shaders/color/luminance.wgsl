// luminance — Rec. 709 relative luminance of a linear-RGB color.
fn luminance(c: vec3<f32>) -> f32 {
  return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}
