// gradient/jet — classic MATLAB jet (not perceptually uniform; use Turbo instead).
fn jet(t: f32) -> vec3<f32> {
  let x = clamp(t, 0.0, 1.0);
  let r = clamp(min(4.0 * x - 1.5, -4.0 * x + 4.5), 0.0, 1.0);
  let g = clamp(min(4.0 * x - 0.5, -4.0 * x + 3.5), 0.0, 1.0);
  let b = clamp(min(4.0 * x + 0.5, -4.0 * x + 2.5), 0.0, 1.0);
  return vec3<f32>(r, g, b);
}
