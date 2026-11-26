// ToonMaterial's shader: cel shading, with the diffuse light of each light cut
// into bands or read from a ramp, a hard highlight and a rim.
//
// Composes the `mesh/transform` and `color/linear_to_srgb` registry snippets,
// then `light.wgsl` and `light_loops.wgsl`, with this main body. Bindings
// (overridable via shader.set):
//   group 0 binding 0: camera   (view_proj, position)
//   group 0 binding 1: lights   (filled by the renderer, see light_loops.wgsl)
//   group 0 binding 2: light_grid (filled by the renderer)
//   group 1 binding 0: material (base color, highlight, rim, bands, alpha
//                                cut-off, alpha_mode flag, and the UV set and
//                                transform of the base color map)
//   group 2 bindings 0..3: the base color map and its sampler, then the ramp
//                          map and its own sampler, so a ramp keeps its own
//                          wrap and filter
//
// Per-Model transform comes through the per-instance vertex attributes at
// locations 3..6, the four columns of `mat4x4<f32>`, as in the PBR shader.
//
// Vertex layout the mesh must provide, the one `Vertex::pbr` and the glTF loader build:
//   @location(0) position : vec3<f32>
//   @location(1) normal   : vec3<f32>
//   @location(2) uv0      : vec2<f32>
//   @location(7) color0   : vec4<f32>, glTF COLOR_0 vertex tint (white default)
//   @location(8) uv1      : vec2<f32>, glTF TEXCOORD_1, sampled when the map is set to UV set 1

struct Camera {
  view_proj: mat4x4<f32>,
  position: vec3<f32>,
}

struct ToonMaterial {
  base_color: vec4<f32>,
  // The hard highlight's color, which black turns off, and the size of its
  // spot: it lights where the half vector's cosine to the normal tops `1 - size`.
  highlight: vec3<f32>,
  highlight_size: f32,
  // The rim's color, which black turns off, and its width: it lights where
  // the view's cosine to the normal stays under `width`.
  rim: vec3<f32>,
  rim_width: f32,
  alpha_cutoff: f32,
  // glTF 2.0 `alphaMode` projected to a numeric flag: 0=Opaque, 1=Mask, 2=Blend.
  // Only `Mask` gates a `discard` in fs_main.
  alpha_mode_flag: u32,
  // The steps the diffuse light takes from the shadow's edge to facing the light.
  bands: u32,
  // Moves `uv` to offset 64, since an array in a uniform starts on 16 bytes.
  // The padding is spelled out because the GL backends, WebGL2 among them,
  // write the struct into a std140 block member by member, without @align and @size.
  pad0: u32,
  // The UV source of the base color map. An array of one keeps the key
  // `material.uv[0]` the PBR shader uses for its base color map.
  uv: array<TextureUv, 1>,
}

// The UVs a map samples: the vertex UV set `uv_set` names (0 or 1),
// through its KHR_texture_transform. `pad` fills the struct to 32 bytes,
// a stride a uniform array can take.
struct TextureUv {
  offset: vec2<f32>,
  scale: vec2<f32>,
  rotation: f32,
  uv_set: u32,
  pad: vec2<u32>,
}

// A map's sampling UV. KHR_texture_transform scales the UV, rotates it, then offsets it.
fn slot_uv(slot: TextureUv, uv0: vec2<f32>, uv1: vec2<f32>) -> vec2<f32> {
  let uv = select(uv0, uv1, slot.uv_set == 1u) * slot.scale;
  let s = sin(slot.rotation);
  let c = cos(slot.rotation);
  return vec2<f32>(c * uv.x + s * uv.y, c * uv.y - s * uv.x) + slot.offset;
}

// Whether the target stores colors encoded for display. The renderer sets it
// from the target's format, as for the PBR shader.
override ENCODE_SRGB: bool = true;

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> material: ToonMaterial;

// A 1x1 white fallback fills each map until the caller sets one.
@group(2) @binding(0) var base_color_map: texture_2d<f32>;
@group(2) @binding(1) var base_color_sampler: sampler;
@group(2) @binding(2) var ramp_map: texture_2d<f32>;
@group(2) @binding(3) var ramp_sampler: sampler;

// What `shade_light` reads at a fragment, which `accumulate_lights` passes it.
struct Surface {
  albedo: vec3<f32>,
}

// The diffuse level, from black to white, of a surface whose normal has the
// cosine `ndl` to the light. A ramp wider than one texel reads it left to
// right, its left half for the faces turned away from the light. Without one,
// the lit side steps up in `bands` even steps and the side turned away gets 0.
fn toon_level(ndl: f32) -> vec3<f32> {
  if (textureDimensions(ramp_map).x > 1u) {
    let u = clamp(ndl * 0.5 + 0.5, 0.0, 1.0);
    // The light loops branch per fragment, where only an explicit level samples.
    return textureSampleLevel(ramp_map, ramp_sampler, vec2<f32>(u, 0.5), 0.0).rgb;
  }
  if (ndl <= 0.0) {
    return vec3<f32>(0.0);
  }
  let steps = f32(max(material.bands, 1u));
  return vec3<f32>(min(ceil(ndl * steps) / steps, 1.0));
}

// The light `lt` sends toward `v` from a surface at `world` with normal `n`:
// the albedo at its toon level, and the highlight where the half vector is
// close enough to the normal. The 1/π matches PbrMaterial's Lambert diffuse,
// so one light shows a toon and a PBR model at the same brightness.
fn shade_light(lt: Light, world: vec3<f32>, n: vec3<f32>, v: vec3<f32>, surface: Surface) -> vec3<f32> {
  let ray = light_ray(lt, world);
  let ndl = dot(n, ray.direction);
  var reflected = surface.albedo * toon_level(ndl);
  let h = normalize(ray.direction + v);
  if (ndl > 0.0 && dot(n, h) > 1.0 - material.highlight_size) {
    reflected = reflected + material.highlight;
  }
  return reflected * ray.radiance * 0.31830988;
}

struct VsOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) world_normal: vec3<f32>,
  @location(2) uv0: vec2<f32>,
  @location(3) vertex_color: vec4<f32>,
  @location(4) uv1: vec2<f32>,
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
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let uv = slot_uv(material.uv[0], in.uv0, in.uv1);
  let base = material.base_color * textureSample(base_color_map, base_color_sampler, uv) * in.vertex_color;
  if (material.alpha_mode_flag == 1u && base.a < material.alpha_cutoff) {
    discard;
  }
  let n = normalize(in.world_normal);
  let v = normalize(camera.position - in.world);
  let lit = accumulate_lights(in.world, n, v, Surface(base.rgb));
  // The scene's ambient light, tinted by the albedo, as on PbrMaterial.
  let ambient = base.rgb * lights.ambient;
  // The rim needs no light: it marks the silhouette from the camera.
  let rim = select(vec3<f32>(0.0), material.rim, dot(n, v) < material.rim_width);
  let color = lit + ambient + rim;
  return vec4<f32>(select(color, linear_to_srgb(color), ENCODE_SRGB), base.a);
}
