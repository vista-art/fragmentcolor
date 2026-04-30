// sample/sphere_uniform — uniform direction on the unit sphere.
fn sphere_uniform(rnd: vec2<f32>) -> vec3<f32> {
  let z = 1.0 - 2.0 * rnd.x;
  let r = sqrt(max(0.0, 1.0 - z * z));
  let phi = 6.28318530718 * rnd.y;
  return vec3<f32>(r * cos(phi), r * sin(phi), z);
}
