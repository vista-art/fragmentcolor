// sample/hemisphere_uniform — uniform hemisphere direction around normal `n`.
fn hemisphere_uniform(rnd: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
  let z = rnd.x;
  let r = sqrt(max(0.0, 1.0 - z * z));
  let phi = 6.28318530718 * rnd.y;
  let tangent = normalize(cross(n, select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(n.y) > 0.99)));
  let bitangent = cross(n, tangent);
  return normalize(tangent * (r * cos(phi)) + bitangent * (r * sin(phi)) + n * z);
}
