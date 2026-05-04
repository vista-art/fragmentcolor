// gradient/viridis — matplotlib's Viridis via IQ cosine palette approximation.
fn viridis(t: f32) -> vec3<f32> {
  let x = clamp(t, 0.0, 1.0);
  let c0 = vec3<f32>(0.274344, 0.004462, 0.331359);
  let c1 = vec3<f32>(0.108915, 1.397291, 1.388110);
  let c2 = vec3<f32>(-0.319631, 0.243490, 0.156419);
  let c3 = vec3<f32>(-4.629188, -5.882480, -19.646180);
  let c4 = vec3<f32>(6.181719, 14.388598, 57.442181);
  let c5 = vec3<f32>(4.876952, -13.955112, -66.125783);
  let c6 = vec3<f32>(-5.513165, 4.473137, 26.855598);
  return c0 + x * (c1 + x * (c2 + x * (c3 + x * (c4 + x * (c5 + x * c6)))));
}
