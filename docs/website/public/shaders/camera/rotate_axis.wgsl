// camera/rotate_axis — Rodrigues 4x4 rotation around arbitrary unit axis `axis`.
fn rotate_axis(axis: vec3<f32>, a: f32) -> mat4x4<f32> {
  let x = axis.x; let y = axis.y; let z = axis.z;
  let s = sin(a); let c = cos(a); let t = 1.0 - c;
  return mat4x4<f32>(
    vec4<f32>(t * x * x + c,     t * x * y + s * z, t * x * z - s * y, 0.0),
    vec4<f32>(t * x * y - s * z, t * y * y + c,     t * y * z + s * x, 0.0),
    vec4<f32>(t * x * z + s * y, t * y * z - s * x, t * z * z + c,     0.0),
    vec4<f32>(0.0,               0.0,               0.0,               1.0)
  );
}
