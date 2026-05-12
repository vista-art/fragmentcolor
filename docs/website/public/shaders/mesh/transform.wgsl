// mesh/transform — local-to-world helpers for per-mesh model matrices.
//
// Self-contained: declares no uniforms and no bind groups. Your main shader
// owns the model matrix (as a uniform, push constant, or per-instance
// attribute, whichever fits the pipeline) and calls these helpers.
//
// Common pairing:
//   var<uniform> mesh_model: mat4x4<f32>;
//   ...
//   let world_pos = mesh_transform_position(in.position, mesh_model);
//   let world_n   = mesh_transform_normal(in.normal,    mesh_model);

fn mesh_transform_position(local: vec3<f32>, model: mat4x4<f32>) -> vec4<f32> {
  return model * vec4<f32>(local, 1.0);
}

fn mesh_transform_direction(local: vec3<f32>, model: mat4x4<f32>) -> vec3<f32> {
  let m3 = mat3x3<f32>(model[0].xyz, model[1].xyz, model[2].xyz);
  return m3 * local;
}

// Best-effort for uniform-scale + rotation + translation transforms. For
// non-uniform scale, pass `transpose(inverse(mat3x3(model)))` instead — that's
// the canonical "normal matrix" and there's no cheap way to derive it in
// shader code without burning per-vertex math.
fn mesh_transform_normal(local: vec3<f32>, model: mat4x4<f32>) -> vec3<f32> {
  return normalize(mesh_transform_direction(local, model));
}
