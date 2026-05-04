// postfx/gaussian_weight_1d — 1D Gaussian weight for offset `x` (in pixels/UV units),
// standard deviation `sigma`. Use to build separable blur kernels on the CPU or uniform side.
fn gaussian_weight_1d(x: f32, sigma: f32) -> f32 {
  let s = max(sigma, 1.0e-5);
  return exp(-(x * x) / (2.0 * s * s)) / (2.50662827463 * s);
}
