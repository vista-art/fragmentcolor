// pattern/trigrid — triangular grid line mask. 1 on line, 0 in cell.
fn trigrid(uv: vec2<f32>, scale: f32, thickness: f32, fw: f32) -> f32 {
  let p = uv * scale;
  let q = vec2<f32>(p.x - p.y * 0.5, p.y * 0.8660254);
  let a = fract(q.x);
  let b = fract(q.y);
  let c = fract(1.0 - a - b);
  let d = min(min(a, b), c);
  return 1.0 - smoothstep(thickness - fw, thickness + fw, d);
}
