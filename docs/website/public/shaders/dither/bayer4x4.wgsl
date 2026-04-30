// dither/bayer4x4 — ordered dither threshold in [0, 1) for integer pixel coord.
fn bayer4x4(p: vec2<i32>) -> f32 {
  let m = array<f32, 16>(
     0.0 / 16.0,  8.0 / 16.0,  2.0 / 16.0, 10.0 / 16.0,
    12.0 / 16.0,  4.0 / 16.0, 14.0 / 16.0,  6.0 / 16.0,
     3.0 / 16.0, 11.0 / 16.0,  1.0 / 16.0,  9.0 / 16.0,
    15.0 / 16.0,  7.0 / 16.0, 13.0 / 16.0,  5.0 / 16.0
  );
  let idx = (p.y & 3) * 4 + (p.x & 3);
  return m[idx];
}
