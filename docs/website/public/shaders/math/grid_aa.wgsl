// grid_aa — anti-aliased grid lines: returns 1.0 near grid lines, 0.0 between.
// `period` is spacing in UV units; `line_width` is half-thickness in the same units.
fn grid_aa(uv: vec2<f32>, period: f32, line_width: f32) -> f32 {
  let g = abs(fract(uv / period - 0.5) - 0.5) * period;
  let line = min(g.x, g.y);
  return 1.0 - smoothstep(0.0, line_width, line);
}
