// postfx/scanlines — horizontal scanline multiplier in [1 - strength, 1]. `lines`
// controls density (lines per UV unit).
fn scanlines(uv: vec2<f32>, lines: f32, strength: f32) -> f32 {
  let s = sin(uv.y * lines * 3.141593 * 2.0);
  return 1.0 - strength * (0.5 - 0.5 * s);
}
