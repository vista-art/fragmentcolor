// rgb_to_hsv — RGB in [0, 1] → (H, S, V) in [0, 1]. Sam Hocevar's branchless form.
fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
  let K = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  let p = mix(vec4<f32>(c.bg, K.wz), vec4<f32>(c.gb, K.xy), step(c.b, c.g));
  let q = mix(vec4<f32>(p.xyw, c.r), vec4<f32>(c.r, p.yzx), step(p.x, c.r));
  let d = q.x - min(q.w, q.y);
  let e = 1.0e-10;
  return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}
