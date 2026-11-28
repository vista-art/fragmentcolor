// The normals the outline pass compares: each model's world-space normal,
// packed from -1..1 into 0..1, with an alpha of 1 wherever a surface covers
// the pixel, so the empty background keeps the clear color's alpha of 0.
//
// Composed after the `mesh/transform` registry snippet. The renderer fills
// `camera` and the per-model columns `model_0` to `model_3`, as it does for
// any Shader drawn as a Model's material.

struct Camera {
  view_proj: mat4x4<f32>,
  position: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexOutput {
  @builtin(position) clip: vec4<f32>,
  @location(0) normal: vec3<f32>,
}

@vertex
fn vs_main(
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(3) model_0: vec4<f32>,
  @location(4) model_1: vec4<f32>,
  @location(5) model_2: vec4<f32>,
  @location(6) model_3: vec4<f32>,
) -> VertexOutput {
  let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
  var out: VertexOutput;
  out.clip = camera.view_proj * mesh_transform_position(position, model);
  out.normal = mesh_transform_normal(normal, model);
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(normalize(in.normal) * 0.5 + 0.5, 1.0);
}
