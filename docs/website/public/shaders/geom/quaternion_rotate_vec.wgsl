// geom/quaternion_rotate_vec — rotate vec3 `v` by unit quaternion `q` (xyzw).
// v' = q * v * q_conjugate, computed in minimized form.
fn quaternion_rotate_vec(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
  let t = 2.0 * cross(q.xyz, v);
  return v + q.w * t + cross(q.xyz, t);
}
