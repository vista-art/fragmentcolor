// sdf2d/hexagon — regular hexagon with circumradius `r`, flat top.
fn hexagon(p: vec2<f32>, r: f32) -> f32 {
  let k = vec3<f32>(-0.8660254, 0.5, 0.57735);
  var q = abs(p);
  q = q - 2.0 * min(dot(k.xy, q), 0.0) * k.xy;
  q = q - vec2<f32>(clamp(q.x, -k.z * r, k.z * r), r);
  return length(q) * sign(q.y);
}
