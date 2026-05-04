// geom/quaternion_slerp — spherical linear interpolation between unit quaternions.
fn quaternion_slerp(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
  var bb = b;
  var cos_theta = dot(a, b);
  if (cos_theta < 0.0) { bb = -b; cos_theta = -cos_theta; }
  if (cos_theta > 0.9995) { return normalize(mix(a, bb, t)); }
  let theta = acos(clamp(cos_theta, -1.0, 1.0));
  let sin_theta = sin(theta);
  let w1 = sin((1.0 - t) * theta) / sin_theta;
  let w2 = sin(t * theta) / sin_theta;
  return a * w1 + bb * w2;
}
