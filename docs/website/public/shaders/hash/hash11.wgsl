// hash11 — 1D → 1D float hash in [0, 1). Good default for procedural noise seeds.
fn hash11(p: f32) -> f32 {
  var x = fract(p * 0.1031);
  x = x * (x + 33.33);
  return fract(x * x);
}
