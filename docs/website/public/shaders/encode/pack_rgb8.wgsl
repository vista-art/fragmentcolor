// encode/pack_rgb8 — pack a unit RGB color (0..1) into a single u32 (0x00BBGGRR layout).
fn pack_rgb8(c: vec3<f32>) -> u32 {
  let q = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
  let r = u32(q.r * 255.0 + 0.5);
  let g = u32(q.g * 255.0 + 0.5);
  let b = u32(q.b * 255.0 + 0.5);
  return r | (g << 8u) | (b << 16u);
}
