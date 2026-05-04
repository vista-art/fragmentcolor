// camera/ray_from_uv — build a world-space ray direction for a UV pixel.
// `uv` in [0, 1], `aspect` = width / height, `fov_y` radians, `view` is world→view basis.
// `view` rows are right (x), up (y), forward (z, looking at -z in view-space).
fn ray_from_uv(uv: vec2<f32>, aspect: f32, fov_y: f32, view: mat3x3<f32>) -> vec3<f32> {
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  let h = tan(fov_y * 0.5);
  let dir_view = normalize(vec3<f32>(ndc.x * aspect * h, ndc.y * h, -1.0));
  return normalize(transpose(view) * dir_view);
}
