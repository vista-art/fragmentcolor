// easing/in_out_quad — quad curve, reflected at t = 0.5.
fn in_out_quad(t: f32) -> f32 {
  return select(1.0 - pow(-2.0 * t + 2.0, 2.0) * 0.5, 2.0 * t * t, t < 0.5);
}
