// camera/scale — 4x4 non-uniform scale matrix.
fn scale(s: vec3<f32>) -> mat4x4<f32> {
  return mat4x4<f32>(
    vec4<f32>(s.x, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, s.y, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, s.z, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
  );
}
