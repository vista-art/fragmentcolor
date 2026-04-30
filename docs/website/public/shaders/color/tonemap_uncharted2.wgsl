// tonemap_uncharted2 — Hable's Uncharted 2 operator (with white-point scale).
fn _u2_partial(x: vec3<f32>) -> vec3<f32> {
  let A = 0.15; let B = 0.50; let C = 0.10;
  let D = 0.20; let E = 0.02; let F = 0.30;
  return ((x * (A * x + vec3<f32>(C * B)) + vec3<f32>(D * E))
       / (x * (A * x + vec3<f32>(B))     + vec3<f32>(D * F))) - vec3<f32>(E / F);
}

fn tonemap_uncharted2(c: vec3<f32>) -> vec3<f32> {
  let W = 11.2;
  let curr = _u2_partial(c * 2.0);
  let white = _u2_partial(vec3<f32>(W));
  return curr / white;
}
