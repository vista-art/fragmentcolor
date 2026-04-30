// rgb_to_hsl — RGB in [0, 1] → (H, S, L) in [0, 1].
fn rgb_to_hsl(c: vec3<f32>) -> vec3<f32> {
  let mx = max(c.r, max(c.g, c.b));
  let mn = min(c.r, min(c.g, c.b));
  let l = (mx + mn) * 0.5;
  let d = mx - mn;
  if (d <= 1.0e-6) { return vec3<f32>(0.0, 0.0, l); }
  let s = select(d / (2.0 - mx - mn), d / (mx + mn), l < 0.5);
  var h = 0.0;
  if (mx == c.r) { h = (c.g - c.b) / d + select(0.0, 6.0, c.g < c.b); }
  else if (mx == c.g) { h = (c.b - c.r) / d + 2.0; }
  else { h = (c.r - c.g) / d + 4.0; }
  return vec3<f32>(h / 6.0, s, l);
}
