// gradient/cubehelix — Dave Green's cubehelix ramp, gentle rotating hue with linear lightness.
// Defaults: start = 0.5, rot = -1.5, hue = 1.0, gamma = 1.0.
fn cubehelix(t: f32, start: f32, rot: f32, hue: f32, gamma: f32) -> vec3<f32> {
  let x = clamp(t, 0.0, 1.0);
  let a = 2.0 * 3.141593 * (start / 3.0 + rot * x);
  let l = pow(x, gamma);
  let amp = hue * l * (1.0 - l) * 0.5;
  let cosa = cos(a);
  let sina = sin(a);
  let r = l + amp * (-0.14861 * cosa + 1.78277 * sina);
  let g = l + amp * (-0.29227 * cosa - 0.90649 * sina);
  let b = l + amp * (1.97294 * cosa);
  return clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0));
}
