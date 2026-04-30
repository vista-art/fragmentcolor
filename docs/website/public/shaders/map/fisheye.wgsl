// map/fisheye — spherical fisheye mapping.
fn fisheye(uv: vec2<f32>, strength: f32) -> vec2<f32> {
  let c = uv * 2.0 - 1.0;
  let r = length(c);
  let theta = r * strength;
  let n = select(c / r, vec2<f32>(0.0), r < 1.0e-5);
  return (n * sin(theta)) * 0.5 + 0.5;
}
