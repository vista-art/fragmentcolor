// camera/screen_to_world — unproject a UV + depth into world coordinates.
fn screen_to_world(uv: vec2<f32>, depth: f32, view_proj_inv: mat4x4<f32>) -> vec3<f32> {
  let ndc = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
  let w = view_proj_inv * ndc;
  return w.xyz / w.w;
}
