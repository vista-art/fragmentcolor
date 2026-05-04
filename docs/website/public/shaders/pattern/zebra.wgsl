// pattern/zebra — wavy zebra stripes driven by a sine of (x + y*warp + t).
// `warp` adds a diagonal skew; `freq` is stripes per unit; `fw` is smoothing width.
fn zebra(uv: vec2<f32>, freq: f32, warp: f32, phase: f32, fw: f32) -> f32 {
  let s = sin((uv.x + uv.y * warp) * freq + phase);
  return smoothstep(-fw, fw, s);
}
