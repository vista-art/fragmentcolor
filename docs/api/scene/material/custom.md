# Material::custom

Wrap an arbitrary `Shader` as a `Material` so it can be paired with a
`Mesh` to form a `Model`. The PBR-named setters (`base_color`, `metallic`,
…) still work; they call `shader.set(...)` under matching uniform paths
(`material.base_color`, etc.). If your shader doesn't declare those
uniforms the setters log-warn and return self unchanged; set custom uniforms
directly via `material.shader().set(...)`.

`Material::custom` is the escape hatch for any shading model that isn't
glTF-spec PBR: toon shading, flat unlit colors, wireframe debug views,
volumetric or post-process effects that you want to attach to a `Mesh`.

## What custom shaders should declare for full Material-style ergonomics

Optional but recommended:

- `struct PbrMaterial { base_color: vec4<f32>, … }` at `@group(1) @binding(0)`
  bound as `material`. This lets the PBR factor setters work as-is.
- A `uv` field in that struct, with one entry per texture slot in
  `TextureSlot` order, for `uv_transform`, `texture_transform` and
  `texture_uv` to write. The built-in PBR shader declares it this way:

```wgsl
struct TextureUv {
  offset: vec2<f32>,
  scale: vec2<f32>,
  rotation: f32,
  @size(12) uv_set: u32,
}

struct PbrMaterial {
  base_color: vec4<f32>,
  // … the other factors …
  @align(16) uv: array<TextureUv, 5>,
}
```

- Four `vec4<f32>` vertex inputs named `model_0` to `model_3`, the columns of
  the Model's transform. The renderer fills them at draw time from each
  Model's `set_transform`, `translate`, `rotate` and `scale`. The PBR shader
  reads them at locations 3 to 6. Any free locations work, since the renderer
  matches these inputs by name. It fills them for each visible Model that a
  pass draws through this shader. A Model's Mesh that no visible Model in the
  pass draws through it, such as a hidden Model's, or one in another pass that
  shares the Material, draws nothing. A shader that draws none of the pass's
  Models and reads these columns for a Model's Mesh, such as this Material's
  shader rendered on its own, makes `Renderer::render` return
  `MeshError::InvalidInstance`.

Without these the Material still draws. The factor setters do nothing, and
every Model draws at its Mesh's own coordinates, whatever its transform.

## Lights

A render shader takes the [Lights](https://fragmentcolor.org/api/scene/light)
that reach it by declaring a root named `lights`, and optionally one named
`light_grid`, at any group and binding. The member names, types and offsets
must match the ones below, and the struct names are free. The renderer fills
both roots in render passes, so they don't appear in `list_uniforms` or
`list_keys`, and `set` and `get` reach only `lights.ambient`.

```wgsl
struct Light {                   // 64 bytes
  kind: u32,                     // 0 directional, 1 point, 2 spot
  intensity: f32,
  range: f32,                    // 0 reaches everywhere
  inner_cone_cos: f32,
  position: vec3<f32>,
  outer_cone_cos: f32,
  direction: vec3<f32>,
  color: vec3<f32>,
}
struct LightGrid {               // 112 bytes
  view_proj: mat4x4<f32>,
  depth_plane: vec4<f32>,
  size: vec3<u32>,
  slicing: u32,                  // 0 no grid, 1 logarithmic, 2 linear
  slice_scale: f32,
  slice_bias: f32,
  reach: f32,
}
struct Lights {
  count: u32,
  global_count: u32,
  promoted_count: u32,
  ambient: vec3<f32>,
  grid: LightGrid,               // at byte 32
  lights: array<Light>,          // at byte 144
}
@group(0) @binding(1) var<storage, read> lights: Lights;
@group(0) @binding(2) var<storage, read> light_grid: array<u32>;
```

A storage `lights` with another layout, or a `light_grid` of another type,
fails `Shader::new` with `ShaderError::TypeMismatch`, which spells out the
layout above.

`lights.lights` holds `lights.count` lights: first the `global_count` lights
without a range, then the lights with one. A shader without `light_grid`
loops over all of them. With it, a fragment reads only the lights that
reach its cluster. `light_grid` starts with `promoted_count` light indices
that every cluster reads, and cluster `c` follows at word
`promoted_count + 2 * c`, as the word where its index list starts and the
length of that list. Copy `light_cell` and `accumulate_lights` below and
write a `shade_light` for your shading model:

```wgsl
const GRID_MARGIN: f32 = 0.015625;
const CELL_EVERY: u32 = 0xffffffffu;
const CELL_GLOBAL: u32 = 0xfffffffeu;

// The cluster that holds `world`, or CELL_GLOBAL past every light's reach,
// or CELL_EVERY with no grid, behind the grid's eye, or outside the grid.
fn light_cell(world: vec3<f32>) -> u32 {
  let grid = lights.grid;
  if (grid.slicing == 0u) {
    return CELL_EVERY;
  }
  let point = vec4<f32>(world, 1.0);
  let depth = dot(grid.depth_plane, point);
  if (depth > grid.reach) {
    return CELL_GLOBAL;
  }
  let clip = grid.view_proj * point;
  if (clip.w <= 0.0) {
    return CELL_EVERY;
  }
  let size = vec3<f32>(grid.size);
  var slice = depth * grid.slice_scale + grid.slice_bias;
  if (grid.slicing == 1u) {
    slice = log2(max(depth, 1.0e-30)) * grid.slice_scale + grid.slice_bias;
  }
  let cell = vec3<f32>((clip.xy / clip.w * 0.5 + 0.5) * size.xy, slice);
  if (any(cell < vec3<f32>(-GRID_MARGIN)) || any(cell > size + GRID_MARGIN)) {
    return CELL_EVERY;
  }
  let index = min(vec3<u32>(max(cell, vec3<f32>(0.0))), grid.size - vec3<u32>(1u));
  return index.x + grid.size.x * (index.y + grid.size.y * index.z);
}

// Every light that reaches `world`, shaded by your `shade_light`.
fn accumulate_lights(world: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
  var total = vec3<f32>(0.0);
  let cell = light_cell(world);
  if (cell == CELL_EVERY) {
    for (var i = 0u; i < lights.count; i++) {
      total += shade_light(lights.lights[i], world, normal);
    }
    return total;
  }
  for (var i = 0u; i < lights.global_count; i++) {
    total += shade_light(lights.lights[i], world, normal);
  }
  if (cell == CELL_GLOBAL) {
    return total;
  }
  for (var k = 0u; k < lights.promoted_count; k++) {
    total += shade_light(lights.lights[light_grid[k]], world, normal);
  }
  let entry = lights.promoted_count + 2u * cell;
  let first = light_grid[entry];
  for (var k = 0u; k < light_grid[entry + 1u]; k++) {
    total += shade_light(lights.lights[light_grid[first + k]], world, normal);
  }
  return total;
}
```

A light with a range adds nothing past it, and a spot light adds nothing
outside its outer cone, so `shade_light` must honor both for the grid to
stay exact. The `Material::pbr` shader's `shade_light` uses the
`KHR_lights_punctual` falloff, `(1 - (d / range)^4)` scaled by the inverse
square distance.

WebGL2 and native OpenGL have no storage buffers in fragment shaders. There
the same source runs with `lights` turned into a uniform block that holds as
many lights as the device allows, and with no grid, so `light_cell` returns
`CELL_EVERY`. A shader that calls `arrayLength` on either root can't run
there, and its first render returns `ShaderError::TypeMismatch`.

### The older interface

The 0.12.6 shape is the older interface. It still fills on every device,
holding up to its own `N` lights, and goes away in 0.13:

```wgsl
struct LightArray { count: u32, ambient: vec3<f32>, lights: array<Light, N> }
@group(0) @binding(1) var<uniform> lights: LightArray;
```

A shader of this shape takes at most `N` lights, and the `add` past them
fails with `PassError::LightCapReached { cap: N }`. A uniform `lights` that
matches neither shape stays an ordinary uniform, which `set` reaches, and
logs a warning.

## Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Shader};

let flat = Shader::new(r#"
    struct Camera { view_proj: mat4x4<f32>, position: vec3<f32> }
    struct Light { kind: u32, intensity: f32, range: f32, inner_cone_cos: f32,
        position: vec3<f32>, outer_cone_cos: f32, direction: vec3<f32>, color: vec3<f32> }
    struct LightGrid { view_proj: mat4x4<f32>, depth_plane: vec4<f32>, size: vec3<u32>,
        slicing: u32, slice_scale: f32, slice_bias: f32, reach: f32 }
    struct Lights { count: u32, global_count: u32, promoted_count: u32,
        ambient: vec3<f32>, grid: LightGrid, lights: array<Light> }
    @group(0) @binding(0) var<uniform> camera: Camera;
    @group(0) @binding(1) var<storage, read> lights: Lights;

    @vertex
    fn vs_main(
        @location(0) p: vec3<f32>,
        @location(3) model_0: vec4<f32>,
        @location(4) model_1: vec4<f32>,
        @location(5) model_2: vec4<f32>,
        @location(6) model_3: vec4<f32>,
    ) -> @builtin(position) vec4<f32> {
        let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
        return camera.view_proj * model * vec4<f32>(p, 1.0);
    }
    @fragment fn fs_main() -> @location(0) vec4<f32> {
        var glow = lights.ambient;
        for (var i = 0u; i < lights.count; i++) {
            glow += lights.lights[i].color * lights.lights[i].intensity * 0.1;
        }
        return vec4<f32>(glow, 1.0);
    }
"#)?;

let flat_mat = Material::custom(flat);
# assert!(flat_mat.shader().list_uniforms().contains(&"camera".to_string()));
# assert!(!flat_mat.shader().list_uniforms().contains(&"lights".to_string()));
# Ok(())
# }
```
