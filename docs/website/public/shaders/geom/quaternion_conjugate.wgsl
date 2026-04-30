// geom/quaternion_conjugate — negate imaginary part of a quaternion (xyzw layout).
fn quaternion_conjugate(q: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(-q.x, -q.y, -q.z, q.w);
}
