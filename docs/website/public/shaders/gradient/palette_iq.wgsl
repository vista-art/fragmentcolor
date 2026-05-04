// gradient/palette_iq — Inigo Quilez cosine palette: a + b*cos(2π*(c*t + d)).
fn palette_iq(t: f32, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
  return a + b * cos(6.28318530718 * (c * t + d));
}
