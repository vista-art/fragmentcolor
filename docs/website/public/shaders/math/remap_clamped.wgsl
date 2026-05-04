// remap_clamped — linear remap from [in_min, in_max] to [out_min, out_max], clamped.
fn remap_clamped(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
  let t = clamp((x - in_min) / (in_max - in_min), 0.0, 1.0);
  return out_min + (out_max - out_min) * t;
}
