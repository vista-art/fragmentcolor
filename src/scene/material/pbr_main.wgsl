// Material::pbr default complete shader.
//
// Composes the `mesh/transform`, `material/pbr` and `color/linear_to_srgb`
// registry snippets, then `light.wgsl` and `pbr_lights.wgsl`, with this main
// body. Bindings (overridable via shader.set):
//   group 0 binding 0: camera   (view_proj, position)
//   group 0 binding 1: lights   (filled by the renderer, see pbr_lights.wgsl)
//   group 0 binding 2: light_grid (filled by the renderer)
//   group 1 binding 0: material (PBR factors, alpha_mode flag, and the UV
//                                set and transform of each texture slot)
//   group 2 bindings 0..5: five glTF PBR-MR maps and a shared sampler
//
// Per-Model transform comes through the **per-instance** vertex attribute
// stream: locations 3..6 carry the four columns of `mat4x4<f32>`, populated
// from the Models `Pass::add` queued into a Pass-owned instance buffer. Sharing a Shader
// across many Models is therefore free (1 pipeline + 1 bind-group set);
// each Model contributes its own row to the instance buffer.
//
// Vertex layout the mesh must provide:
//   @location(0) position : vec3<f32>
//   @location(1) normal   : vec3<f32>
//   @location(2) uv0      : vec2<f32>
//   @location(7) color0   : vec4<f32>, glTF COLOR_0 vertex tint (white default)
//   @location(8) uv1      : vec2<f32>, glTF TEXCOORD_1, sampled by the
//                                       texture slots set to UV set 1
//   @location(9) tangent  : vec4<f32>, glTF TANGENT. `xyz` is the tangent
//                                       direction in object space, and `w`
//                                       is the bitangent sign (±1) for the
//                                       TBN handedness

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
  // `Blend` ignore the flag, since pipeline state (depth write, blend
  // equation) sets their behavior rather than fragment-shader logic.
  alpha_mode_flag: u32,
  // Move `uv` to offset 64, since an array in a uniform starts on 16 bytes.
  // The padding is spelled out because the GL backends, WebGL2 among them,
  // write the struct into a std140 block member by member, without @align and @size.
  pad0: u32,
  pad1: u32,
  pad2: u32,
  // The UV source of each texture slot, in the order base color,
  // metallic-roughness, normal, occlusion, emissive.
  uv: array<TextureUv, 5>,
}

// The UVs a texture slot samples: the vertex UV set `uv_set` names (0 or 1),
// through its KHR_texture_transform. `pad` fills the struct to 32 bytes,
// a stride a uniform array can take.
struct TextureUv {
  offset: vec2<f32>,
  scale: vec2<f32>,
  rotation: f32,
  uv_set: u32,
  pad: vec2<u32>,
}

// A slot's sampling UV. KHR_texture_transform scales the UV, rotates it,
// then offsets it. A positive rotation turns the UVs counter-clockwise in
// texture space, where v points down, as the extension's example and the
// Khronos sample renderer do.
fn slot_uv(slot: TextureUv, uv0: vec2<f32>, uv1: vec2<f32>) -> vec2<f32> {
  let uv = select(uv0, uv1, slot.uv_set == 1u) * slot.scale;
  let s = sin(slot.rotation);
  let c = cos(slot.rotation);
  return vec2<f32>(c * uv.x + s * uv.y, c * uv.y - s * uv.x) + slot.offset;
}

// Whether the target stores colors encoded for display. The renderer sets it
// from the target's format: true for Rgba8Unorm and Bgra8Unorm, false for an
// sRGB format, which encodes on write, and for a float format, which keeps
// linear light for the passes after it.
override ENCODE_SRGB: bool = true;

@group(0) @binding(0) var<uniform> camera: Camera;
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
  @location(2) uv0: vec2<f32>,
  @location(3) vertex_color: vec4<f32>,
  @location(4) uv1: vec2<f32>,
  // World-space tangent in .xyz, and the bitangent sign in .w, the spec's
  // handedness flag: `B = cross(N, T) * tangent.w`.
  @location(5) world_tangent: vec4<f32>,
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
  @location(7) color0: vec4<f32>,
  @location(8) uv1: vec2<f32>,
  @location(9) tangent: vec4<f32>,
) -> VsOut {
  let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
  var out: VsOut;
  let world = mesh_transform_position(position, model);
  out.clip = camera.view_proj * world;
  out.world = world.xyz;
  out.world_normal = mesh_transform_normal(normal, model);
  out.uv0 = uv0;
  out.vertex_color = color0;
  out.uv1 = uv1;
  // Tangents are direction vectors, transformed by the model matrix's
  // upper 3×3 with no inverse transpose, and the bitangent sign in .w
  // stays untouched.
  let t_world = (model * vec4<f32>(tangent.xyz, 0.0)).xyz;
  out.world_tangent = vec4<f32>(t_world, tangent.w);
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  // Each slot samples its own UV set through its own transform.
  let base_color_uv = slot_uv(material.uv[0], in.uv0, in.uv1);
  let metallic_roughness_uv = slot_uv(material.uv[1], in.uv0, in.uv1);
  let normal_uv = slot_uv(material.uv[2], in.uv0, in.uv1);
  let occlusion_uv = slot_uv(material.uv[3], in.uv0, in.uv1);
  let emissive_uv = slot_uv(material.uv[4], in.uv0, in.uv1);

  // Base color (albedo / F0 source) = factor × map sample × vertex tint.
  // glTF `COLOR_0` is a per-vertex linear-RGB(A) multiplier, defaults to
  // white so the existing factor × map product is preserved when the
  // mesh doesn't carry vertex colors.
  let base_color_sample = textureSample(base_color_map, pbr_sampler, base_color_uv);
  let albedo = material.base_color.rgb * base_color_sample.rgb * in.vertex_color.rgb;
  let alpha = material.base_color.a * base_color_sample.a * in.vertex_color.a;

  // Mask alpha mode: discard fragments below the cut-off before doing any
  // lighting work. Cheap; runs before the BRDF. Opaque and Blend ignore
  // this branch, since pipeline state (depth write and blend equation)
  // sets their behavior, not fragment-shader logic. glTF 2.0 compares the
  // factor × sampled alpha against `material.alpha_cutoff`.
  if (material.alpha_mode_flag == 1u && alpha < material.alpha_cutoff) {
    discard;
  }

  // glTF spec: B encodes metallic, G encodes roughness. The `.bgr` swizzle
  // puts metallic in .r and roughness in .g of the local vector. R is unused
  // by the spec; some authoring tools stash ambient occlusion in it (the
  // "ORM" texture packing convention). Occlusion reads `occlusion_map`
  // instead, so the unused channel here is harmless.
  let mr = textureSample(metallic_roughness_map, pbr_sampler, metallic_roughness_uv).bgr;
  let metallic = material.metallic * mr.r;
  let roughness = material.roughness * mr.g;

  // Normal mapping via the full TBN transform. Decode the stored `[0, 1]`
  // byte triple into a `[-1, 1]` tangent-space normal, scale the XY by
  // `material.normal_scale`, then rotate it from tangent into world space
  // through `(T, B, N)`. The 1×1 default normal map encodes
  // `(128, 128, 255)` → tangent-space `(0, 0, 1)` → final `world_normal`
  // collapses to the geometric `N`, so unset maps don't perturb shading.
  // glTF spec: `B = cross(N, T) * tangent.w` (handedness preserved
  // through model transform + the `tangent.w` sign).
  let n_sample = textureSample(normal_map, pbr_sampler, normal_uv).xyz * 2.0 - 1.0;
  let tn = vec3<f32>(n_sample.xy * material.normal_scale, n_sample.z);
  let n = normalize(in.world_normal);
  let t = normalize(in.world_tangent.xyz);
  let b = cross(n, t) * in.world_tangent.w;
  let world_normal = normalize(t * tn.x + b * tn.y + n * tn.z);

  // Occlusion. glTF reads only the red channel; blend toward `1.0` by the
  // strength factor so `strength = 0` ignores the map entirely.
  let ao_sample = textureSample(occlusion_map, pbr_sampler, occlusion_uv).r;
  let ao = mix(1.0, ao_sample, material.occlusion_strength);

  // Emissive = factor * map sample.
  let emissive_sample = textureSample(emissive_map, pbr_sampler, emissive_uv).rgb;
  let emissive_color = material.emissive * emissive_sample;

  let v = normalize(camera.position - in.world);

  // Direct light from every light that reaches the fragment, see pbr_lights.wgsl.
  let lit_total = accumulate_lights(in.world, world_normal, v, albedo, metallic, roughness);
  // The scene's ambient light, tinted by the albedo,
  // keeps unlit faces from reading pitch-black.
  let ambient = albedo * lights.ambient;
  let color = (lit_total + ambient) * ao + emissive_color;
  return vec4<f32>(select(color, linear_to_srgb(color), ENCODE_SRGB), alpha);
}
