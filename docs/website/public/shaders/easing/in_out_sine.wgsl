// easing/in_out_sine — -(cos(pi t) - 1) / 2.
fn in_out_sine(t: f32) -> f32 { return -(cos(3.141593 * t) - 1.0) * 0.5; }
