// encode/unpack_unorm_4x8 — inverse of pack_unorm_4x8.
fn unpack_unorm_4x8(p: u32) -> vec4<f32> {
  return vec4<f32>(
    f32(p & 0xFFu),
    f32((p >>  8u) & 0xFFu),
    f32((p >> 16u) & 0xFFu),
    f32((p >> 24u) & 0xFFu)
  ) / 255.0;
}
