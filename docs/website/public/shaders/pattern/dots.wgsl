// pattern/dots — circular dots on a square grid. `cells` per UV unit, `radius` in cell units.
fn dots(uv: vec2<f32>, cells: f32, radius: f32, fw: f32) -> f32 {
  let g = fract(uv * cells) - vec2<f32>(0.5);
  return 1.0 - smoothstep(radius - fw, radius + fw, length(g));
}
