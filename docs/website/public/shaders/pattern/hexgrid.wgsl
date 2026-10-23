// pattern/hexgrid — pointy-top hex grid, `scale` cells per UV unit across.
// Returns (edge distance, hex id x, hex id y). The edge distance is in cell widths.
// The id is integer valued: y is the row, x counts half cells, so x + y is always even.
fn hexgrid(uv: vec2<f32>, scale: f32) -> vec3<f32> {
  let s = vec2<f32>(1.0, 1.7320508);
  let p = uv * scale;
  let a = ((p % s) + s) % s - s * 0.5;
  let b = (((p - s * 0.5) % s) + s) % s - s * 0.5;
  let gv = select(b, a, dot(a, a) < dot(b, b));
  let id = floor(2.0 * (p - gv) / s + 0.5);
  let edge = 0.5 - max(abs(gv.x), abs(gv.x) * 0.5 + abs(gv.y) * 0.8660254);
  return vec3<f32>(edge, id.x, id.y);
}
