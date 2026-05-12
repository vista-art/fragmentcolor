// Material::pbr default complete shader.
//
// Built from the `mesh/transform` and `material/pbr` registry snippets plus
// this main body. Bindings (overridable via shader.set):
//   group 0 binding 0 — camera (view_proj, position)
//   group 0 binding 1 — light  (directional: direction, color)
//   group 1 binding 0 — mesh   (model matrix; per-Model)
//   group 1 binding 1 — material (PBR factors)
//
// Vertex layout the mesh must provide, in order:
//   @location(0) position : vec3<f32>
//   @location(1) normal   : vec3<f32>
//   @location(2) uv0      : vec2<f32>

struct Camera {
  view_proj: mat4x4<f32>,
  position: vec3<f32>,
}

struct MeshTransform {
  model: mat4x4<f32>,
}

struct PbrMaterial {
  base_color: vec4<f32>,
  emissive: vec3<f32>,
  metallic: f32,
  roughness: f32,
  normal_scale: f32,
  occlusion_strength: f32,
  alpha_cutoff: f32,
}

struct DirectionalLight {
  direction: vec3<f32>,
  color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> light: DirectionalLight;
@group(1) @binding(0) var<uniform> mesh: MeshTransform;
@group(1) @binding(1) var<uniform> material: PbrMaterial;

struct VsOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) world_normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
}

// Vertex inputs are declared as flat function arguments rather than a struct
// so FragmentColor's Naga-driven reflection sees them as top-level @location
// bindings — that's how `Shader::validate_mesh` matches mesh attributes to
// shader inputs.
@vertex
fn vs_main(
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv0: vec2<f32>,
) -> VsOut {
  var out: VsOut;
  let world = mesh_transform_position(position, mesh.model);
  out.clip = camera.view_proj * world;
  out.world = world.xyz;
  out.world_normal = mesh_transform_normal(normal, mesh.model);
  out.uv = uv0;
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let albedo = material.base_color.rgb;
  let v = normalize(camera.position - in.world);
  let l = normalize(-light.direction);
  let lit = pbr_shade(
    in.world_normal, l, v,
    albedo, material.metallic, material.roughness,
    light.color,
  );
  // Cheap ambient term so unlit faces don't read as pitch-black. A real scene
  // would replace this with image-based lighting; that's a Phase 2 follow-up.
  let ambient = albedo * 0.03;
  let color = lit + ambient + material.emissive;
  return vec4<f32>(color, material.base_color.a);
}
