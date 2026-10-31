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

// A surface normal in world space, right under any scale, even one that
// differs per axis. The normal matrix is the inverse transpose of the model's
// upper 3x3. Its cofactor matrix, whose columns are the cross products of the
// model's column pairs, points the same way, and the sign of the determinant
// keeps a mirrored model's normals facing out.
fn mesh_transform_normal(local: vec3<f32>, model: mat4x4<f32>) -> vec3<f32> {
  let x = model[0].xyz;
  let y = model[1].xyz;
  let z = model[2].xyz;
  let cofactor = mat3x3<f32>(cross(y, z), cross(z, x), cross(x, y));
  return normalize(cofactor * local) * sign(dot(x, cross(y, z)));
}
