// raymarch/normal_from_sdf_taps — estimate a surface normal from 6 central-difference
// distance samples around a point. The caller evaluates its scene SDF at (p ± e·axis)
// and passes in the resulting scalar distances.
fn normal_from_sdf_taps(d_px: f32, d_nx: f32, d_py: f32, d_ny: f32, d_pz: f32, d_nz: f32) -> vec3<f32> {
  return normalize(vec3<f32>(d_px - d_nx, d_py - d_ny, d_pz - d_nz));
}
