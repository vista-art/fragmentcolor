// geom/tbn_from_normal — build a TBN basis from a unit normal, picking an arbitrary tangent.
fn tbn_from_normal(n: vec3<f32>) -> mat3x3<f32> {
  let up = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(n.y) > 0.99);
  let t = normalize(cross(up, n));
  let b = cross(n, t);
  return mat3x3<f32>(t, b, n);
}
