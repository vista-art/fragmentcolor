// sdf/plane — signed distance to a plane with unit normal `n` and offset `h` (n·p + h).
fn plane(p: vec3<f32>, n: vec3<f32>, h: f32) -> f32 {
  return dot(p, normalize(n)) + h;
}
