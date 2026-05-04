// map/rotate2 — rotate a 2D point by `a` radians around the origin.
fn rotate2(p: vec2<f32>, a: f32) -> vec2<f32> {
  let s = sin(a); let c = cos(a);
  return vec2<f32>(c * p.x - s * p.y, s * p.x + c * p.y);
}
