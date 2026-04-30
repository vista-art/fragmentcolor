// sdf/octahedron — regular octahedron with half-diagonal `s`. Exact SDF.
fn octahedron(p: vec3<f32>, s: f32) -> f32 {
  let q = abs(p);
  let m = q.x + q.y + q.z - s;
  var r = vec3<f32>(0.0);
  if (3.0 * q.x < m) { r = q.xyz; }
  else if (3.0 * q.y < m) { r = q.yzx; }
  else if (3.0 * q.z < m) { r = q.zxy; }
  else { return m * 0.57735027; }
  let k = clamp(0.5 * (r.z - r.y + s), 0.0, s);
  return length(vec3<f32>(r.x, r.y - s + k, r.z - k));
}
