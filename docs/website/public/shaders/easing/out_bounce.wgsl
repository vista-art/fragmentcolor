// easing/out_bounce — standard Penner bounce-out.
fn out_bounce(t: f32) -> f32 {
  let n1 = 7.5625;
  let d1 = 2.75;
  if (t < 1.0 / d1) {
    return n1 * t * t;
  } else if (t < 2.0 / d1) {
    let u = t - 1.5 / d1;
    return n1 * u * u + 0.75;
  } else if (t < 2.5 / d1) {
    let u = t - 2.25 / d1;
    return n1 * u * u + 0.9375;
  } else {
    let u = t - 2.625 / d1;
    return n1 * u * u + 0.984375;
  }
}
