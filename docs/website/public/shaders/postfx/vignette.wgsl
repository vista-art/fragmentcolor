// postfx/vignette — multiplier in [0, 1] that darkens edges. Multiply against color.
fn vignette(uv: vec2<f32>, radius: f32, softness: f32) -> f32 {
  let d = length(uv - vec2<f32>(0.5));
  return 1.0 - smoothstep(radius, radius + softness, d);
}
