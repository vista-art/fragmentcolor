// postfx/film_grain — additive grain scalar. Multiply by strength and add to color.
// `seed` decorrelates across frames (use time).
fn film_grain(uv: vec2<f32>, seed: f32) -> f32 {
  var p3 = fract(vec3<f32>(uv.x, uv.y, seed) * vec3<f32>(0.1031, 0.1030, 0.0973));
  p3 = p3 + dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z) * 2.0 - 1.0;
}
