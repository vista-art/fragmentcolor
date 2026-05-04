// line_aa — anti-aliased horizontal/vertical line at `pos` in 1D, width `w` (half-width).
// Returns 1.0 on the line, 0.0 far away, smooth transition within `w`.
fn line_aa(pos: f32, x: f32, w: f32) -> f32 {
  return 1.0 - smoothstep(0.0, w, abs(x - pos));
}
