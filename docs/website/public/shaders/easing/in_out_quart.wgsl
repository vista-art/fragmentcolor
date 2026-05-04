// easing/in_out_quart — quart curve reflected at t = 0.5.
fn in_out_quart(t: f32) -> f32 {
  return select(1.0 - pow(-2.0 * t + 2.0, 4.0) * 0.5, 8.0 * t * t * t * t, t < 0.5);
}
