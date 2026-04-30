// encode/unpack_normal_octahedron — inverse of pack_normal_octahedron.
fn unpack_normal_octahedron(e: vec2<f32>) -> vec3<f32> {
  var n = vec3<f32>(e.x, e.y, 1.0 - abs(e.x) - abs(e.y));
  if (n.z < 0.0) {
    let t = n.xy;
    n = vec3<f32>((vec2<f32>(1.0) - abs(t.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), t >= vec2<f32>(0.0)), n.z);
  }
  return normalize(n);
}
