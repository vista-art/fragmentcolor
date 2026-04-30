// pattern/weave — simple over/under weave pattern. Returns 0 (valley) or 1 (ridge)
// smoothed by `fw`. `cells` controls thread density.
fn weave(uv: vec2<f32>, cells: f32, fw: f32) -> f32 {
  let p = uv * cells;
  let sx = sin(p.x * 3.141593);
  let sy = sin(p.y * 3.141593);
  let m = sx * sy;
  return smoothstep(-fw, fw, m);
}
