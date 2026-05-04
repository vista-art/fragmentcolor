// map/ripple — radial ripple emanating from `center`. `amp` scales UV displacement.
fn ripple(uv: vec2<f32>, center: vec2<f32>, freq: f32, phase: f32, amp: f32) -> vec2<f32> {
  let d = uv - center;
  let r = length(d);
  let n = select(d / r, vec2<f32>(0.0), r < 1.0e-5);
  return uv + n * sin(r * freq - phase) * amp;
}
