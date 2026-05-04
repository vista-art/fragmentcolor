// hsv_to_rgb — H, S, V in [0, 1] → RGB in [0, 1].
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
  let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  let p = abs(fract(vec3<f32>(hsv.x) + K.xyz) * 6.0 - vec3<f32>(K.w));
  return hsv.z * mix(vec3<f32>(K.x), clamp(p - vec3<f32>(K.x), vec3<f32>(0.0), vec3<f32>(1.0)), hsv.y);
}
