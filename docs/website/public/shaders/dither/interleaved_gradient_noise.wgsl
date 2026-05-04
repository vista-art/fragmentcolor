// dither/interleaved_gradient_noise — Jorge Jimenez's IGN (aka "blue-noise-ish").
// Input in integer or fractional pixel coords; output in [0, 1).
fn interleaved_gradient_noise(p: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(0.06711056 * p.x + 0.00583715 * p.y));
}
