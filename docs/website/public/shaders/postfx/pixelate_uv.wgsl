// postfx/pixelate_uv — snap UV to a grid of `cells` cells per unit side.
// Use as a pre-sample hook: sample your texture with the returned UV.
fn pixelate_uv(uv: vec2<f32>, cells: vec2<f32>) -> vec2<f32> {
  return (floor(uv * cells) + vec2<f32>(0.5)) / cells;
}
