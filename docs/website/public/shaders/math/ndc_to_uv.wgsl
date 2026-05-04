// ndc_to_uv — NDC (-1..1, y up) to UV (0..1, y down).
fn ndc_to_uv(ndc: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}
