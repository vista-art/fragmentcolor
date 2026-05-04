// sample/vogel_disk — k-th of `n` Vogel sunflower disk samples; even coverage w/ golden angle.
fn vogel_disk(k: u32, n: u32) -> vec2<f32> {
  let golden = 2.39996323;
  let r = sqrt((f32(k) + 0.5) / f32(n));
  let theta = f32(k) * golden;
  return r * vec2<f32>(cos(theta), sin(theta));
}
