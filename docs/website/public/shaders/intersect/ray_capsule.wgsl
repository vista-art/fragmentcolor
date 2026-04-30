// intersect/ray_capsule — ray vs capsule between endpoints pa, pb with radius r.
// Returns nearest positive t or -1 on miss.
fn ray_capsule(ro: vec3<f32>, rd: vec3<f32>, pa: vec3<f32>, pb: vec3<f32>, r: f32) -> f32 {
  let ba = pb - pa;
  let oa = ro - pa;
  let baba = dot(ba, ba);
  let bard = dot(ba, rd);
  let baoa = dot(ba, oa);
  let rdoa = dot(rd, oa);
  let oaoa = dot(oa, oa);
  let a = baba - bard * bard;
  let b = baba * rdoa - baoa * bard;
  let c = baba * oaoa - baoa * baoa - r * r * baba;
  let h = b * b - a * c;
  if (h >= 0.0) {
    let t = (-b - sqrt(h)) / a;
    let y = baoa + t * bard;
    if (y > 0.0 && y < baba) { return t; }
    let oc = select(ro - pb, ro - pa, y <= 0.0);
    let b2 = dot(rd, oc);
    let c2 = dot(oc, oc) - r * r;
    let h2 = b2 * b2 - c2;
    if (h2 > 0.0) { return -b2 - sqrt(h2); }
  }
  return -1.0;
}
