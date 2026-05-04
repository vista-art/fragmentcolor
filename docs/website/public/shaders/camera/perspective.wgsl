// camera/perspective — right-handed perspective projection, mapping view-space to clip.
// `fov_y` radians, `aspect` = width / height.
fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> mat4x4<f32> {
  let f = 1.0 / tan(fov_y * 0.5);
  let rd = 1.0 / (near - far);
  return mat4x4<f32>(
    vec4<f32>(f / aspect, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, f, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, (near + far) * rd, -1.0),
    vec4<f32>(0.0, 0.0, 2.0 * far * near * rd, 0.0)
  );
}
