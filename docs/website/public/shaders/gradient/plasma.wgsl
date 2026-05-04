// gradient/plasma — matplotlib's Plasma polynomial fit.
fn plasma(t: f32) -> vec3<f32> {
  let x = clamp(t, 0.0, 1.0);
  let c0 = vec3<f32>(0.058732, 0.023314, 0.543754);
  let c1 = vec3<f32>(2.176514, 0.239129, 2.389780);
  let c2 = vec3<f32>(-2.689460, -7.455851, 3.217525);
  let c3 = vec3<f32>(6.130348, 26.092608, -16.864775);
  let c4 = vec3<f32>(-11.107428, -28.872960, 26.669366);
  let c5 = vec3<f32>(10.025790, 13.965727, -15.653994);
  let c6 = vec3<f32>(-3.658344, -3.991907, 3.297927);
  return c0 + x * (c1 + x * (c2 + x * (c3 + x * (c4 + x * (c5 + x * c6)))));
}
