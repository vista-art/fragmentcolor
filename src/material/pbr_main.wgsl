// Material::pbr default complete shader.
//
// Composes the `mesh/transform` and `material/pbr` registry snippets with
// this main body. Bindings (overridable via shader.set):
//   group 0 binding 0 — camera   (view_proj, position)
//   group 0 binding 1 — lights   (LightArray: count + ambient + up to
//                                 PBR_MAX_LIGHTS KHR_lights_punctual lights)
//   group 1 binding 0 — material (PBR factors + alpha_mode flag)
//   group 2 bindings 0..5 — five glTF PBR-MR maps + shared sampler
//
// Per-Model transform comes through the **per-instance** vertex attribute
// stream — locations 3..6 carry the four columns of `mat4x4<f32>`, populated
// by `Pass::add_model` into a Pass-owned instance buffer. Sharing a Shader
// across many Models is therefore free (1 pipeline + 1 bind-group set);
// each Model contributes its own row to the instance buffer.
//
// Vertex layout the mesh must provide:
//   @location(0) position : vec3<f32>
//   @location(1) normal   : vec3<f32>
//   @location(2) uv0      : vec2<f32>
//   @location(7) color0   : vec4<f32>  — glTF COLOR_0 vertex tint (white default)
//   @location(8) uv1      : vec2<f32>  — glTF TEXCOORD_1, used by maps that
//                                        opt into the second UV set (the
//                                        per-map `texCoord` selector lands
//                                        with KHR_texture_transform)
//   @location(9) tangent  : vec4<f32>  — glTF TANGENT; `xyz` is the tangent
//                                        direction in object space, `w` is
//                                        the bitangent sign (±1) for the
//                                        TBN handedness

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
  // KHR_texture_transform — single global UV transform applied to every
  // map's sampling UV. Spec-correct per-map transforms are a follow-up;
  // most glTF assets only carry KHR_texture_transform on `base_color`
  // anyway, and the global path covers that case losslessly when the
  // other maps share the same transform (or are absent).
  uv_offset: vec2<f32>,
  uv_scale: vec2<f32>,
  uv_rotation: f32,
}

// glTF KHR_lights_punctual model. Each entry in `lights.lights[i]` is one
// light; `kind` selects which fields fs_main consults:
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

// Fixed-size light list. `count` is the number of valid entries in
// `lights[..count]`; `ambient` is a scene-wide RGB term added to the final
// shaded color so unlit faces don't read as pitch-black (replaces the
// hardcoded `albedo * 0.03` from the prior single-light shader).
//
// `PBR_MAX_LIGHTS = 32` keeps the uniform near 2 KiB (1 u32 + vec3
// ambient + 32 × Light(64) = 32 + 2048 = 2080 bytes), well inside wgpu's
// per-binding limit. Beyond a few dozen lights the forward-shaded loop
// becomes the bottleneck; a clustered / storage-buffer path is on the
// roadmap for scenes that need hundreds.
const PBR_MAX_LIGHTS: u32 = 32u;

struct LightArray {
  count: u32,
  ambient: vec3<f32>,
  lights: array<Light, 32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> lights: LightArray;
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
  @location(3) vertex_color: vec4<f32>,
  @location(4) uv1: vec2<f32>,
  // World-space tangent in .xyz; bitangent sign in .w (the spec's
  // handedness flag — `B = cross(N, T) * tangent.w`).
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
  out.uv = uv0;
  out.vertex_color = color0;
  out.uv1 = uv1;
  // Tangents are direction vectors, transformed by the model matrix's
  // upper-3×3 (no inverse-transpose) — the bitangent sign in .w stays
  // untouched.
  let t_world = (model * vec4<f32>(tangent.xyz, 0.0)).xyz;
  out.world_tangent = vec4<f32>(t_world, tangent.w);
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  // KHR_texture_transform — single global UV transform applied to every
  // map sample. Scale → rotate → offset matches the spec's transform
  // composition. Defaults (scale = [1,1], rotation = 0, offset = [0,0])
  // map every UV to itself so unset materials sample as before.
  let scaled = in.uv * material.uv_scale;
  let s = sin(material.uv_rotation);
  let c = cos(material.uv_rotation);
  let rotated = vec2<f32>(scaled.x * c - scaled.y * s, scaled.x * s + scaled.y * c);
  let uv = rotated + material.uv_offset;

  // Base color (albedo / F0 source) = factor × map sample × vertex tint.
  // glTF `COLOR_0` is a per-vertex linear-RGB(A) multiplier, defaults to
  // white so the existing factor × map product is preserved when the
  // mesh doesn't carry vertex colors.
  let base_color_sample = textureSample(base_color_map, pbr_sampler, uv);
  let albedo = material.base_color.rgb * base_color_sample.rgb * in.vertex_color.rgb;
  let alpha = material.base_color.a * base_color_sample.a * in.vertex_color.a;

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
  let mr = textureSample(metallic_roughness_map, pbr_sampler, uv).bgr;
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
  let n_sample = textureSample(normal_map, pbr_sampler, uv).xyz * 2.0 - 1.0;
  let tn = vec3<f32>(n_sample.xy * material.normal_scale, n_sample.z);
  let n = normalize(in.world_normal);
  let t = normalize(in.world_tangent.xyz);
  let b = cross(n, t) * in.world_tangent.w;
  let world_normal = normalize(t * tn.x + b * tn.y + n * tn.z);

  // Occlusion. glTF reads only the red channel; blend toward `1.0` by the
  // strength factor so `strength = 0` ignores the map entirely.
  let ao_sample = textureSample(occlusion_map, pbr_sampler, uv).r;
  let ao = mix(1.0, ao_sample, material.occlusion_strength);

  // Emissive = factor * map sample.
  let emissive_sample = textureSample(emissive_map, pbr_sampler, uv).rgb;
  let emissive_color = material.emissive * emissive_sample;

  let v = normalize(camera.position - in.world);

  // Accumulate direct lighting from every active light. The KHR_lights_
  // punctual loop dispatches on `kind` per fragment:
  //   directional → unattenuated parallel rays from `-direction`
  //   point       → inverse-square distance from `position`, optional
  //                 (1 - (d/range)^4) cutoff
  //   spot        → point + smooth cone falloff between inner / outer
  //                 cone cosines, cone axis = `-direction`
  var lit_total: vec3<f32> = vec3<f32>(0.0);
  let n_lights = min(lights.count, PBR_MAX_LIGHTS);
  for (var i: u32 = 0u; i < n_lights; i = i + 1u) {
    let lt = lights.lights[i];
    var l: vec3<f32>;
    var attenuation: f32 = 1.0;
    if (lt.kind == 0u) {
      l = normalize(-lt.direction);
    } else {
      let to_light = lt.position - in.world;
      let dist = length(to_light);
      l = to_light / max(dist, 1.0e-6);
      attenuation = 1.0 / max(dist * dist, 1.0e-4);
      if (lt.range > 0.0) {
        attenuation = attenuation * max(0.0, 1.0 - pow(dist / lt.range, 4.0));
      }
      if (lt.kind == 2u) {
        let spot_axis = normalize(-lt.direction);
        let cos_angle = dot(spot_axis, l);
        attenuation = attenuation * smoothstep(lt.outer_cone_cos, lt.inner_cone_cos, cos_angle);
      }
    }
    let radiance = lt.color * lt.intensity * attenuation;
    lit_total = lit_total + pbr_shade(
      world_normal, l, v,
      albedo, metallic, roughness,
      radiance,
    );
  }
  // Scene-wide ambient — keeps unlit faces from reading pitch-black.
  // Drives an albedo-tinted constant; future image-based lighting will
  // replace this branch.
  let ambient = albedo * lights.ambient;
  let color = (lit_total + ambient) * ao + emissive_color;
  return vec4<f32>(color, alpha);
}
