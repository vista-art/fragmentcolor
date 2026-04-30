// sdf/hexagonal_prism — hex prism with (radius, half-height) in `h`.
fn hexagonal_prism(p: vec3<f32>, h: vec2<f32>) -> f32 {
  let k = vec3<f32>(-0.8660254, 0.5, 0.57735);
  var q = abs(p);
  q = vec3<f32>(q.x - 2.0 * min(dot(k.xy, q.xy), 0.0) * k.x,
                q.y - 2.0 * min(dot(k.xy, q.xy), 0.0) * k.y,
                q.z);
  let d = vec2<f32>(length(q.xy - vec2<f32>(clamp(q.x, -k.z * h.x, k.z * h.x), h.x)) * sign(q.y - h.x),
                    q.z - h.y);
  return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0)));
}
