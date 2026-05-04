// pattern/brick — staggered brick grid (running-bond). 1 inside brick body, 0 on mortar lines.
fn brick(uv: vec2<f32>, cells: vec2<f32>, mortar: f32, fw: f32) -> f32 {
  var p = uv * cells;
  if ((i32(floor(p.y)) & 1) == 1) { p.x = p.x + 0.5; }
  let f = fract(p);
  let edge = min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y));
  return smoothstep(mortar - fw, mortar + fw, edge);
}
