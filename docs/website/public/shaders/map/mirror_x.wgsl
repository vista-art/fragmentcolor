// map/mirror_x — fold UV across x = 0.5. Useful for horizontal symmetry.
fn mirror_x(uv: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(0.5 - abs(uv.x - 0.5), uv.y);
}
