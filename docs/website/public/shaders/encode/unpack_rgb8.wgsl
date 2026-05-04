// encode/unpack_rgb8 — inverse of pack_rgb8 (0x00BBGGRR → vec3 in [0,1]).
fn unpack_rgb8(p: u32) -> vec3<f32> {
  let r = f32(p & 0xFFu);
  let g = f32((p >> 8u) & 0xFFu);
  let b = f32((p >> 16u) & 0xFFu);
  return vec3<f32>(r, g, b) / 255.0;
}
