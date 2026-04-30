// gradient/inferno — matplotlib's Inferno polynomial fit.
fn inferno(t: f32) -> vec3<f32> {
  let x = clamp(t, 0.0, 1.0);
  let c0 = vec3<f32>(0.000217, 0.000159, -0.014574);
  let c1 = vec3<f32>(0.130285, 0.573944, 3.144131);
  let c2 = vec3<f32>(11.604230, -3.485475, -14.089730);
  let c3 = vec3<f32>(-41.703432, 17.436495, 55.180000);
  let c4 = vec3<f32>(76.712750, -33.402827, -85.664450);
  let c5 = vec3<f32>(-73.200470, 32.706155, 58.736528);
  let c6 = vec3<f32>(27.018149, -12.641907, -17.295950);
  return c0 + x * (c1 + x * (c2 + x * (c3 + x * (c4 + x * (c5 + x * c6)))));
}
