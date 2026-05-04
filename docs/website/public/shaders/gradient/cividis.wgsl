// gradient/cividis — matplotlib's Cividis (color-vision-deficient friendly) fit.
fn cividis(t: f32) -> vec3<f32> {
  let x = clamp(t, 0.0, 1.0);
  let c0 = vec3<f32>(0.000000, 0.135112, 0.304751);
  let c1 = vec3<f32>(-0.028369, 1.050469, 1.482660);
  let c2 = vec3<f32>(-0.197778, -1.471267, -4.811950);
  let c3 = vec3<f32>(0.987010, 3.268300, 10.197900);
  let c4 = vec3<f32>(-1.571630, -2.951960, -9.553800);
  let c5 = vec3<f32>(1.810900, 1.259520, 3.887700);
  return c0 + x * (c1 + x * (c2 + x * (c3 + x * (c4 + x * c5))));
}
