// map/scale2 — non-uniform 2D scale about a pivot.
fn scale2(p: vec2<f32>, pivot: vec2<f32>, s: vec2<f32>) -> vec2<f32> {
  return (p - pivot) * s + pivot;
}
