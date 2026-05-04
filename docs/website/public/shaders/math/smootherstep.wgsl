// smootherstep — Ken Perlin's 2nd-order smoothstep: 6t^5 - 15t^4 + 10t^3.
// Zero 1st and 2nd derivatives at both endpoints.
fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
  let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
  return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}
