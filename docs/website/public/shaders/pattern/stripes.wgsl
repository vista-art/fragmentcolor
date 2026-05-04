// pattern/stripes — straight stripes. 1 on-stripe, 0 off. `period` in UV units,
// `duty` fraction of period that is "on". Anti-aliased via smoothstep on `fw` width.
fn stripes(uv: vec2<f32>, direction: vec2<f32>, period: f32, duty: f32, fw: f32) -> f32 {
  let t = dot(uv, normalize(direction)) / period;
  let f = fract(t);
  return 1.0 - smoothstep(duty - fw, duty + fw, f);
}
