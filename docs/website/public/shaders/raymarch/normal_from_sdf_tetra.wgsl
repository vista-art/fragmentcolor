// raymarch/normal_from_sdf_tetra — cheaper 4-tap tetrahedral normal from SDF samples.
// Pass the scene-SDF values at the four tetrahedral offsets (k1..k4). Directions:
//   k1 = ( 1, -1, -1), k2 = (-1, -1,  1), k3 = (-1,  1, -1), k4 = ( 1,  1,  1).
fn normal_from_sdf_tetra(k1: f32, k2: f32, k3: f32, k4: f32) -> vec3<f32> {
  let n = vec3<f32>( 1.0, -1.0, -1.0) * k1
        + vec3<f32>(-1.0, -1.0,  1.0) * k2
        + vec3<f32>(-1.0,  1.0, -1.0) * k3
        + vec3<f32>( 1.0,  1.0,  1.0) * k4;
  return normalize(n);
}
