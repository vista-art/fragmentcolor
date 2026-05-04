// sample/disk_concentric — Shirley's concentric mapping from [0,1]^2 to disk,
// preserves stratification better than polar sqrt warp.
fn disk_concentric(rnd: vec2<f32>) -> vec2<f32> {
  let u = rnd * 2.0 - vec2<f32>(1.0);
  if (u.x == 0.0 && u.y == 0.0) { return vec2<f32>(0.0); }
  var r: f32;
  var theta: f32;
  if (abs(u.x) > abs(u.y)) {
    r = u.x;
    theta = 0.7853982 * (u.y / u.x);
  } else {
    r = u.y;
    theta = 1.5707963 - 0.7853982 * (u.x / u.y);
  }
  return r * vec2<f32>(cos(theta), sin(theta));
}
