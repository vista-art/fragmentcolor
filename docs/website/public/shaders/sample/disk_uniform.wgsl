// sample/disk_uniform — uniform sample on the unit disk via sqrt warp.
fn disk_uniform(rnd: vec2<f32>) -> vec2<f32> {
  let r = sqrt(rnd.x);
  let phi = 6.28318530718 * rnd.y;
  return vec2<f32>(r * cos(phi), r * sin(phi));
}
