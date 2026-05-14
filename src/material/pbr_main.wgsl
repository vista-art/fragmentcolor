// Material::pbr default complete shader.
//
// Composes the `mesh/transform` and `material/pbr` registry snippets with
// this main body. Bindings (overridable via shader.set):
//   group 0 binding 0 — camera   (view_proj, position)
//   group 0 binding 1 — light    (KHR_lights_punctual shape: kind, direction,
//                                 position, color, intensity, range, cone cos)
//   group 1 binding 0 — material (PBR factors + alpha_mode flag)
//   group 2 bindings 0..5 — five glTF PBR-MR maps + shared sampler
//
// Per-Model transform comes through the **per-instance** vertex attribute
// stream — locations 3..6 carry the four columns of `mat4x4<f32>`, populated
// by `Pass::add_model` into a Pass-owned instance buffer. Sharing a Shader
// across many Models is therefore free (1 pipeline + 1 bind-group set);
// each Model contributes its own row to the instance buffer.
//
// Vertex layout the mesh must provide, in order:
//   @location(0) position : vec3<f32>
//   @location(1) normal   : vec3<f32>
//   @location(2) uv0      : vec2<f32>

struct Camera {
  view_proj: mat4x4<f32>,
  position: vec3<f32>,
}

struct PbrMaterial {
  base_color: vec4<f32>,
  emissive: vec3<f32>,
  metallic: f32,
  roughness: f32,
  normal_scale: f32,
  occlusion_strength: f32,
  alpha_cutoff: f32,
  // glTF 2.0 `alphaMode` projected to a numeric flag: 0=Opaque, 1=Mask,
  // 2=Blend. Only `Mask` gates a `discard` in fs_main; `Opaque` and
  // `Blend` ignore the flag — their behaviour is pipeline-state
  // (depth-write, blend equation) rather than fragment-shader logic.
  alpha_mode_flag: u32,
}

// glTF KHR_lights_punctual model. One binding holds the active light; `kind`
// selects which fields fs_main consults:
//   0 = directional — parallel rays, only `direction` matters
//   1 = point       — inverse-square distance from `position`, optional
//                     `range` cutoff (0 = unlimited)
//   2 = spot        — point + cone falloff via `inner_cone_cos` /
//                     `outer_cone_cos`, cone axis = `-direction`
// `intensity` scales `color` uniformly so glTF's per-light intensity slots in
// without Rust-side premultiplication.
struct Light {
  kind: u32,
  intensity: f32,
  range: f32,
  inner_cone_cos: f32,
  position: vec3<f32>,
  outer_cone_cos: f32,
  direction: vec3<f32>,
  color: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> light: Light;
@group(1) @binding(0) var<uniform> material: PbrMaterial;

// glTF 2.0 PBR-MR texture maps. Every Material::pbr starts with a 1×1
// fallback bound in each slot (see Renderer::default_pbr_textures) so the
// bind group is always complete; user code overrides any slot via
// `Material::base_color_texture(&tex)` and friends.
@group(2) @binding(0) var base_color_map: texture_2d<f32>;
@group(2) @binding(1) var metallic_roughness_map: texture_2d<f32>;
@group(2) @binding(2) var normal_map: texture_2d<f32>;
@group(2) @binding(3) var occlusion_map: texture_2d<f32>;
@group(2) @binding(4) var emissive_map: texture_2d<f32>;
@group(2) @binding(5) var pbr_sampler: sampler;

struct VsOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) world_normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv0: vec2<f32>,
  @location(3) model_0: vec4<f32>,
  @location(4) model_1: vec4<f32>,
  @location(5) model_2: vec4<f32>,
  @location(6) model_3: vec4<f32>,
) -> VsOut {
  let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
  var out: VsOut;
  let world = mesh_transform_position(position, model);
  out.clip = camera.view_proj * world;
  out.world = world.xyz;
  out.world_normal = mesh_transform_normal(normal, model);
  out.uv = uv0;
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  // Base color (albedo / F0 source) = factor * map sample.
  let base_color_sample = textureSample(base_color_map, pbr_sampler, in.uv);
  let albedo = material.base_color.rgb * base_color_sample.rgb;
  let alpha = material.base_color.a * base_color_sample.a;

  // Mask alpha mode: discard fragments below the cut-off before doing any
  // lighting work. Cheap; runs before the BRDF. Opaque and Blend ignore
  // this branch — their semantics live in pipeline state (depth-write +
  // blend equation), not fragment-shader logic. glTF 2.0 compares the
  // factor × sampled alpha against `material.alpha_cutoff`.
  if (material.alpha_mode_flag == 1u && alpha < material.alpha_cutoff) {
    discard;
  }

  // glTF spec: B encodes metallic, G encodes roughness. The `.bgr` swizzle
  // puts metallic in .r and roughness in .g of the local vector. R is unused
  // by the spec; some authoring tools stash ambient occlusion in it (the
  // "ORM" texture packing convention) — we read it from `occlusion_map`
  // instead, so the unused channel here is harmless.
  let mr = textureSample(metallic_roughness_map, pbr_sampler, in.uv).bgr;
  let metallic = material.metallic * mr.r;
  let roughness = material.roughness * mr.g;

  // Normal mapping. Decode the stored `[0, 1]` byte triple into a
  // `[-1, 1]` tangent-space normal, scale the XY perturbation by
  // `material.normal_scale`, and add it additively to the interpolated
  // world-space normal. This is a placeholder combine that proves the
  // binding works while the full tangent-space-to-world TBN transform is
  // finished as a follow-up. The 1×1 default `(128, 128, 255)` decodes to
  // `(0, 0, 1)` → scaled XY is `(0, 0)` → addition is a no-op, so unset
  // maps don't perturb the lit normal.
  let n_sample = textureSample(normal_map, pbr_sampler, in.uv).xyz * 2.0 - 1.0;
  let n_perturb = vec3<f32>(n_sample.xy * material.normal_scale, 0.0);
  let world_normal = normalize(in.world_normal + n_perturb);

  // Occlusion. glTF reads only the red channel; blend toward `1.0` by the
  // strength factor so `strength = 0` ignores the map entirely.
  let ao_sample = textureSample(occlusion_map, pbr_sampler, in.uv).r;
  let ao = mix(1.0, ao_sample, material.occlusion_strength);

  // Emissive = factor * map sample.
  let emissive_sample = textureSample(emissive_map, pbr_sampler, in.uv).rgb;
  let emissive_color = material.emissive * emissive_sample;

  let v = normalize(camera.position - in.world);

  // Per-fragment light direction `l` (surface → light) and attenuation.
  // Directional lights are unattenuated parallel rays; point and spot use
  // inverse-square distance with optional range cutoff; spot adds a smooth
  // cone falloff between the inner and outer cone cosines.
  var l: vec3<f32>;
  var attenuation: f32 = 1.0;
  if (light.kind == 0u) {
    l = normalize(-light.direction);
  } else {
    let to_light = light.position - in.world;
    let dist = length(to_light);
    l = to_light / max(dist, 1.0e-6);
    attenuation = 1.0 / max(dist * dist, 1.0e-4);
    if (light.range > 0.0) {
      attenuation = attenuation * max(0.0, 1.0 - pow(dist / light.range, 4.0));
    }
    if (light.kind == 2u) {
      let spot_axis = normalize(-light.direction);
      let cos_angle = dot(spot_axis, l);
      attenuation = attenuation * smoothstep(light.outer_cone_cos, light.inner_cone_cos, cos_angle);
    }
  }
  let radiance = light.color * light.intensity * attenuation;
  let lit = pbr_shade(
    world_normal, l, v,
    albedo, metallic, roughness,
    radiance,
  );
  // Cheap ambient term so unlit faces don't read as pitch-black. Real scenes
  // would replace this with image-based lighting; that's a Phase 2 follow-up.
  let ambient = albedo * 0.03;
  let color = (lit + ambient) * ao + emissive_color;
  return vec4<f32>(color, alpha);
}
