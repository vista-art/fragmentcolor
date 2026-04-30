// geom/quaternion_from_axis_angle — unit quaternion (xyzw) from axis + angle.
fn quaternion_from_axis_angle(axis: vec3<f32>, angle: f32) -> vec4<f32> {
  let h = angle * 0.5;
  let s = sin(h);
  return vec4<f32>(normalize(axis) * s, cos(h));
}
