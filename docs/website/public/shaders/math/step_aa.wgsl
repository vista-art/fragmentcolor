// step_aa — screen-space anti-aliased step using analytic derivative width.
// Usage: `let m = step_aa(threshold, value, fwidth_proxy);` where fwidth_proxy
// approximates the pixel-space spread (e.g. length(vec2(dpdx(value), dpdy(value)))).
fn step_aa(edge: f32, x: f32, w: f32) -> f32 {
  return smoothstep(edge - w, edge + w, x);
}
