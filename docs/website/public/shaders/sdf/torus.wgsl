// sdf/torus — XZ-plane torus; t.x is major radius, t.y is tube (minor) radius.
fn torus(p: vec3<f32>, t: vec2<f32>) -> f32 {
  let q = vec2<f32>(length(p.xz) - t.x, p.y);
  return length(q) - t.y;
}
