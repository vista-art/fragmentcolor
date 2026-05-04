// easing/in_back — overshoot before start. Classic Robert Penner coefficients.
fn in_back(t: f32) -> f32 {
  let c1 = 1.70158;
  let c3 = c1 + 1.0;
  return c3 * t * t * t - c1 * t * t;
}
