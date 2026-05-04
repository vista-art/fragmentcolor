// map/polar — (uv-center) → (angle/TAU, radius). Handy for radial patterns.
fn polar(uv: vec2<f32>, center: vec2<f32>) -> vec2<f32> {
  let c = uv - center;
  let r = length(c);
  let a = atan2(c.y, c.x) / 6.28318530718;
  return vec2<f32>(fract(a + 1.0), r);
}
