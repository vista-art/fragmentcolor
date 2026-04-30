// postfx/crt_curvature — CRT-style screen curvature. Returns warped UV plus an
// out-of-bounds flag (negative if outside screen) packed into .z of a vec3.
fn crt_curvature(uv: vec2<f32>, amount: f32) -> vec3<f32> {
  var c = uv * 2.0 - 1.0;
  let off = c.yx * c.yx * amount;
  c = c + c * off;
  let warped = c * 0.5 + 0.5;
  let oob = min(min(warped.x, 1.0 - warped.x), min(warped.y, 1.0 - warped.y));
  return vec3<f32>(warped, oob);
}
