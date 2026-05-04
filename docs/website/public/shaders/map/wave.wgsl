// map/wave — horizontal sine displacement of UVs; `amp` is in UV units, `freq` in cycles/unit.
fn wave(uv: vec2<f32>, freq: f32, phase: f32, amp: f32) -> vec2<f32> {
  return vec2<f32>(uv.x + amp * sin(uv.y * freq + phase), uv.y);
}
