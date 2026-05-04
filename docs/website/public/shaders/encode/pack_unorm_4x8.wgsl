// encode/pack_unorm_4x8 — pack a vec4 in [0,1] into a single u32 (wgsl built-in friendly).
fn pack_unorm_4x8(c: vec4<f32>) -> u32 {
  let q = clamp(c, vec4<f32>(0.0), vec4<f32>(1.0));
  let r = u32(q.r * 255.0 + 0.5);
  let g = u32(q.g * 255.0 + 0.5);
  let b = u32(q.b * 255.0 + 0.5);
  let a = u32(q.a * 255.0 + 0.5);
  return r | (g << 8u) | (b << 16u) | (a << 24u);
}
