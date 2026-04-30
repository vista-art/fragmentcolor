// pattern/hexgrid — distance to nearest hex edge (useful as a line mask or hex indexing).
// Returns (min edge dist, hex_id.x, hex_id.y) as vec3 so callers can color-per-cell if desired.
fn hexgrid(uv: vec2<f32>, scale: f32) -> vec3<f32> {
  let s = vec2<f32>(1.0, 1.7320508);
  let p = uv * scale;
  let a = ((p % s) + s) % s - s * 0.5;
  let b = (((p - s * 0.5) % s) + s) % s - s * 0.5;
  let gv = select(b, a, dot(a, a) < dot(b, b));
  let id = p - gv;
  let edge = 0.5 - max(abs(gv.x), abs(gv.y) * 0.5 + abs(gv.x) * 0.5);
  return vec3<f32>(edge, id.x, id.y);
}
