// pcg2d — Mark Jarzynski's 2D PCG hash, u32x2 → u32x2. High-quality, cheap.
fn pcg2d(v: vec2<u32>) -> vec2<u32> {
  var p = v * vec2<u32>(1664525u) + vec2<u32>(1013904223u);
  p.x = p.x + p.y * 1664525u;
  p.y = p.y + p.x * 1664525u;
  p = p ^ (p >> vec2<u32>(16u));
  p.x = p.x + p.y * 1664525u;
  p.y = p.y + p.x * 1664525u;
  p = p ^ (p >> vec2<u32>(16u));
  return p;
}
