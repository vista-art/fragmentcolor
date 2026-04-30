// map/tile — tile UVs by `cells` per unit side. Returns UVs in [0, 1) per cell.
fn tile(uv: vec2<f32>, cells: vec2<f32>) -> vec2<f32> {
  return fract(uv * cells);
}
