// sdf2d/star — `n`-pointed star; `r` is outer radius, `m` in (2, n) sets sharpness.
fn star(p: vec2<f32>, r: f32, n: u32, m: f32) -> f32 {
  let an = 3.141593 / f32(n);
  let en = 3.141593 / m;
  let acs = vec2<f32>(cos(an), sin(an));
  let ecs = vec2<f32>(cos(en), sin(en));
  let bn = (atan2(abs(p.x), p.y) - an * floor((atan2(abs(p.x), p.y)) / an * 0.5) * 2.0) - an;
  var q = length(p) * vec2<f32>(cos(bn), abs(sin(bn)));
  q = q - r * acs;
  q = q + ecs * clamp(-dot(q, ecs), 0.0, r * acs.y / ecs.y);
  return length(q) * sign(q.x);
}
