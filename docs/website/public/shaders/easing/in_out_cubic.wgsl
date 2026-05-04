// easing/in_out_cubic — cubic reflected at t = 0.5.
fn in_out_cubic(t: f32) -> f32 {
  return select(1.0 - pow(-2.0 * t + 2.0, 3.0) * 0.5, 4.0 * t * t * t, t < 0.5);
}
