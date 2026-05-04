// pcg3d — Mark Jarzynski's 3D PCG hash, u32x3 → u32x3.
fn pcg3d(v: vec3<u32>) -> vec3<u32> {
  var p = v * vec3<u32>(1664525u) + vec3<u32>(1013904223u);
  p.x = p.x + p.y * p.z;
  p.y = p.y + p.z * p.x;
  p.z = p.z + p.x * p.y;
  p = p ^ (p >> vec3<u32>(16u));
  p.x = p.x + p.y * p.z;
  p.y = p.y + p.z * p.x;
  p.z = p.z + p.x * p.y;
  return p;
}
