// rsqrt — reciprocal sqrt, with a tiny guard to avoid Inf for zero inputs.
fn rsqrt(x: f32) -> f32 {
  return 1.0 / sqrt(max(x, 1.0e-20));
}
