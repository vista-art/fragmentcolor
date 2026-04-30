// smooth_pulse — soft pulse with smoothstep edges of width `w` on each side.
fn smooth_pulse(a: f32, b: f32, w: f32, x: f32) -> f32 {
  return smoothstep(a - w, a + w, x) - smoothstep(b - w, b + w, x);
}
