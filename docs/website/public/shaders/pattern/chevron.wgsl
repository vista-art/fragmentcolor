// pattern/chevron — V-shaped stripes. `period` vertical spacing, `slant` horizontal lean.
fn chevron(uv: vec2<f32>, period: f32, slant: f32, fw: f32) -> f32 {
  let y = uv.y + abs(uv.x - floor(uv.x + 0.5)) * slant;
  let f = fract(y / period);
  return 1.0 - smoothstep(0.5 - fw, 0.5 + fw, f);
}
