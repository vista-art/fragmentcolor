// remap — linear remap from [in_min, in_max] to [out_min, out_max]. Not clamped.
fn remap(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
  return out_min + (out_max - out_min) * (x - in_min) / (in_max - in_min);
}
