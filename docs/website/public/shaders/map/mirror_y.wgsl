// map/mirror_y — fold UV across y = 0.5. Useful for vertical symmetry.
fn mirror_y(uv: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(uv.x, 0.5 - abs(uv.y - 0.5));
}
