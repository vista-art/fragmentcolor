// sdf/op_elongate — stretches an SDF along axes by `h`. Returns displaced point.
fn op_elongate(p: vec3<f32>, h: vec3<f32>) -> vec3<f32> {
  return p - clamp(p, -h, h);
}
