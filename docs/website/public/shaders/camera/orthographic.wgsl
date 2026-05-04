// camera/orthographic — right-handed orthographic projection.
fn orthographic(l: f32, r: f32, b: f32, t: f32, near: f32, far: f32) -> mat4x4<f32> {
  let rl = 1.0 / (r - l);
  let tb = 1.0 / (t - b);
  let fn_ = 1.0 / (far - near);
  return mat4x4<f32>(
    vec4<f32>(2.0 * rl, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 2.0 * tb, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, -fn_, 0.0),
    vec4<f32>(-(r + l) * rl, -(t + b) * tb, -near * fn_, 1.0)
  );
}
