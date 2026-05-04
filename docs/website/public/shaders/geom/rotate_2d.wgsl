// geom/rotate_2d — 2x2 rotation matrix.
fn rotate_2d(a: f32) -> mat2x2<f32> {
  let s = sin(a); let c = cos(a);
  return mat2x2<f32>(vec2<f32>(c, s), vec2<f32>(-s, c));
}
