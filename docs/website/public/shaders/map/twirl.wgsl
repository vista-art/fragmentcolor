// map/twirl — UV twirl around `center`. Falls off smoothly with distance.
fn twirl(uv: vec2<f32>, center: vec2<f32>, strength: f32) -> vec2<f32> {
  let d = uv - center;
  let r = length(d);
  let a = atan2(d.y, d.x) + strength * (1.0 - smoothstep(0.0, 0.5, r));
  return center + r * vec2<f32>(cos(a), sin(a));
}
