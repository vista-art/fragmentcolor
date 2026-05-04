// camera/translate — 4x4 translation matrix.
fn translate(t: vec3<f32>) -> mat4x4<f32> {
  return mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(t.x, t.y, t.z, 1.0)
  );
}
