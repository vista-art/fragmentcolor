// hsl_to_rgb — H, S, L in [0, 1] → linear-space RGB in [0, 1].
fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
  let h = hsl.x; let s = hsl.y; let l = hsl.z;
  let c = (1.0 - abs(2.0 * l - 1.0)) * s;
  let hp = h * 6.0;
  let x = c * (1.0 - abs((hp - 2.0 * floor(hp * 0.5)) - 1.0));
  var rgb = vec3<f32>(0.0);
  if (hp < 1.0) { rgb = vec3<f32>(c, x, 0.0); }
  else if (hp < 2.0) { rgb = vec3<f32>(x, c, 0.0); }
  else if (hp < 3.0) { rgb = vec3<f32>(0.0, c, x); }
  else if (hp < 4.0) { rgb = vec3<f32>(0.0, x, c); }
  else if (hp < 5.0) { rgb = vec3<f32>(x, 0.0, c); }
  else               { rgb = vec3<f32>(c, 0.0, x); }
  return rgb + vec3<f32>(l - 0.5 * c);
}
