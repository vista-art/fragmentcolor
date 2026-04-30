// encode/pack_normal_octahedron — unit vec3 → vec2 in [-1, 1]^2 via octahedron mapping.
// Pairs with unpack_normal_octahedron. Good for 2x16f normal storage.
fn pack_normal_octahedron(n: vec3<f32>) -> vec2<f32> {
  let nn = n / (abs(n.x) + abs(n.y) + abs(n.z));
  var e = nn.xy;
  if (nn.z < 0.0) {
    e = (vec2<f32>(1.0) - abs(e.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), e >= vec2<f32>(0.0));
  }
  return e;
}
