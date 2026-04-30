// encode/morton2d — interleave bits of (x, y) into a single u32 (Z-order curve).
fn _spread_bits_2d(x: u32) -> u32 {
  var v = x & 0x0000FFFFu;
  v = (v | (v << 8u)) & 0x00FF00FFu;
  v = (v | (v << 4u)) & 0x0F0F0F0Fu;
  v = (v | (v << 2u)) & 0x33333333u;
  v = (v | (v << 1u)) & 0x55555555u;
  return v;
}

fn morton2d(x: u32, y: u32) -> u32 {
  return _spread_bits_2d(x) | (_spread_bits_2d(y) << 1u);
}
