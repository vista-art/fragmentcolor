// dither/bayer2x2 — ordered dither threshold in [0, 1) for integer pixel coord.
fn bayer2x2(p: vec2<i32>) -> f32 {
  let m = array<f32, 4>(0.0, 0.5, 0.75, 0.25);
  let idx = (p.y & 1) * 2 + (p.x & 1);
  return m[idx];
}
