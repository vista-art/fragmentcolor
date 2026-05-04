// checker — classic checkerboard 0/1 based on `cells` cells per unit.
fn checker(uv: vec2<f32>, cells: f32) -> f32 {
  let g = floor(uv * cells);
  return (g.x + g.y) - 2.0 * floor((g.x + g.y) * 0.5);
}
