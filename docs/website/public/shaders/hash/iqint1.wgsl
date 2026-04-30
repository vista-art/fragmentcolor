// iqint1 — Inigo Quilez's simple integer hash, u32 → u32 → f32 in [0,1).
fn iqint1(n: u32) -> f32 {
  var x = n;
  x = (x << 13u) ^ x;
  x = x * (x * x * 15731u + 789221u) + 1376312589u;
  return f32(x & 0x7fffffffu) / f32(0x7fffffffu);
}
