// sdf2d/rounded_box — rectangle with per-corner radii (x = tl, y = tr, z = br, w = bl-ish:
// matches iq's convention of (+x+y, -x+y, -x-y, +x-y)).
fn rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
  var rr = vec2<f32>(r.x, r.y);
  if (p.x <= 0.0) { rr = vec2<f32>(r.z, r.w); }
  let rc = select(rr.y, rr.x, p.y > 0.0);
  let q = abs(p) - b + vec2<f32>(rc);
  return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - rc;
}
