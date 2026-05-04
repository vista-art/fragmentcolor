// gradient/spectral — ColorBrewer-style rainbow (not perceptually uniform).
// Polynomial fit suitable for quick diverging/qualitative displays.
fn spectral(t: f32) -> vec3<f32> {
  let x = clamp(t, 0.0, 1.0);
  let c0 = vec3<f32>(0.619, 0.002, 0.259);
  let c1 = vec3<f32>(-0.071, 3.085, 1.058);
  let c2 = vec3<f32>(6.411, -3.432, -0.837);
  let c3 = vec3<f32>(-17.500, 15.240, 9.900);
  let c4 = vec3<f32>(19.240, -18.760, -25.100);
  let c5 = vec3<f32>(-6.950, 4.260, 15.650);
  return clamp(c0 + x * (c1 + x * (c2 + x * (c3 + x * (c4 + x * c5)))), vec3<f32>(0.0), vec3<f32>(1.0));
}
