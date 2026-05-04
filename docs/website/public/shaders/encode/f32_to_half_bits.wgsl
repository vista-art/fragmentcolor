// encode/f32_to_half_bits — convert a single-precision float to IEEE 754 half (f16) bits in low 16 of u32.
// Handles normals, denormals, zero, infinity; NaN becomes quiet NaN.
fn f32_to_half_bits(x: f32) -> u32 {
  let b = bitcast<u32>(x);
  let sign = (b >> 16u) & 0x8000u;
  var exp_ = i32((b >> 23u) & 0xFFu) - 127 + 15;
  var mant = b & 0x7FFFFFu;
  if (exp_ >= 0x1F) {
    if (mant != 0u) { return sign | 0x7E00u; } // NaN
    return sign | 0x7C00u;                     // Inf
  } else if (exp_ <= 0) {
    if (exp_ < -10) { return sign; }
    mant = (mant | 0x800000u) >> u32(1 - exp_ + 13);
    return sign | mant;
  }
  return sign | (u32(exp_) << 10u) | (mant >> 13u);
}
