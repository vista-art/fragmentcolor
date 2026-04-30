// easing/out_quad — 1 - (1 - t)^2.
fn out_quad(t: f32) -> f32 { return 1.0 - (1.0 - t) * (1.0 - t); }
