// sample/halton — 1D low-discrepancy Halton sample for index `i` and prime `base`.
fn halton(i: u32, base: u32) -> f32 {
  var f = 1.0; var r = 0.0; var n = i;
  while (n > 0u) {
    f = f / f32(base);
    r = r + f * f32(n % base);
    n = n / base;
  }
  return r;
}
