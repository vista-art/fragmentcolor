// camera/look_at — right-handed look-at view matrix (world → view).
fn look_at(eye: vec3<f32>, at: vec3<f32>, up: vec3<f32>) -> mat4x4<f32> {
  let f = normalize(at - eye);
  let s = normalize(cross(f, up));
  let u = cross(s, f);
  return mat4x4<f32>(
    vec4<f32>( s.x,  u.x, -f.x, 0.0),
    vec4<f32>( s.y,  u.y, -f.y, 0.0),
    vec4<f32>( s.z,  u.z, -f.z, 0.0),
    vec4<f32>(-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0)
  );
}
