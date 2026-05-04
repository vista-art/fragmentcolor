// postfx/chromatic_offsets — returns three UVs to sample R, G, B at, given a
// base UV and a radial strength. Feed the outputs to three texture samples.
fn chromatic_offsets(uv: vec2<f32>, strength: f32) -> mat3x2<f32> {
  let c = uv - vec2<f32>(0.5);
  let d = c * strength;
  return mat3x2<f32>(uv + d, uv, uv - d);
}
