# Shader

The [Shader](https://fragmentcolor.org/api/core/shader) object is the main building block in [FragmentColor](https://fragmentcolor.org).

It takes a WGSL or GLSL shader source as input, parses it, validates it, and exposes the uniforms as keys.

`Shader::new` refuses a source that no device FragmentColor opens can run, with a `ShaderError` that names the problem: a feature outside the set FragmentColor requests (such as `f16` or `f64`), two globals on one `@group` and `@binding` (a `lights` root included), writable storage in a `@vertex` entry point, an immediate root that breaks the uniform layout rules, a filtering `sampler` that samples a depth texture, and two vertex inputs with one name, which would read one mesh attribute. A shader that passes can still be one a given device can't run, which `Renderer::render` refuses, as [Device limits](https://fragmentcolor.org/api/core/shader#device-limits) describes.

`Shader::new` accepts a source string, a registry slug like `"sdf2d/circle"`, an `https://` URL, a local file path, or an array mixing any of those. Array parts are deduplicated by hash and concatenated in order, so you can pull pure helper functions from the public registry at `https://fragmentcolor.org/shaders/` into your own shader without copy-pasting. Override the registry base with [Shader::set_registry](https://fragmentcolor.org/api/core/shader#set_registry).

To draw your shader, you must use your [Shader](https://fragmentcolor.org/api/core/shader) instance as input to a [Renderer](https://fragmentcolor.org/api/core/renderer).

You can compose [Shader](https://fragmentcolor.org/api/core/shader) instances into a [Pass](https://fragmentcolor.org/api/core/pass) object to create more complex rendering pipelines.

You can also create renderings with multiple Render Passes by passing an array of [Pass](https://fragmentcolor.org/api/core/pass) instances to [Renderer::render](https://fragmentcolor.org/api/core/renderer#render).

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Shader, Renderer};

let shader = Shader::new(r#"
    @vertex
    fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
        var pos = array<vec2<f32>, 3>(
            vec2<f32>(-1.0, -1.0),
            vec2<f32>( 3.0, -1.0),
            vec2<f32>(-1.0,  3.0)
        );
        return vec4<f32>(pos[index], 0.0, 1.0);
    }

    @group(0) @binding(0)
    var<uniform> resolution: vec2<f32>;

    @fragment
    fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red
    }
"#)?;

// Set the "resolution" uniform
shader.set("resolution", [800.0, 600.0])?;
let res: [f32; 2] = shader.get("resolution")?;

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
renderer.render(&shader, &target)?;

# assert_eq!(res, [800.0, 600.0]);
# assert!(shader.list_uniforms().len() >= 1);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

## Uniforms

Classic uniforms are declared with `var<uniform>` and can be nested structs/arrays.
FragmentColor exposes every root and nested field as addressable keys using dot and index notation:

- Set a field: `shader.set("u.color", [r, g, b, a])`
- Index arrays: `shader.set("u.arr[1]", value)`

### WGSL example

```wgsl
struct MyUniform { 
  color: vec4<f32>, 
  arr: array<vec4<f32>, 2> 
};

@group(0) @binding(0) var<uniform> u: MyUniform;
```

FragmentColor handles std140-style 16-byte alignment for you, and large uniform blobs are pooled internally. There is nothing to configure.

## Textures and Samplers

Sampled textures and samplers are supported via `texture_*` and `sampler` declarations.
You can bind a Texture object created by the Renderer directly to a texture uniform (e.g., `shader.set("tex", &texture)`);
samplers are provided automatically:

- If a texture is bound in the same group, the sampler defaults to that texture's sampler.
- Otherwise, a reasonable default sampler is used.

### WGSL example

```wgsl
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
```

1D, 2D and 3D textures bind with the view dimension the WGSL declaration gives. A FragmentColor texture's view always covers all of it, so a cube, cube array or 2D array slot takes no texture yet, and `Renderer::render` refuses one with `RendererError::BindingError`. Integer textures map to `Sint` / `Uint` sample types; float textures use filterable float when the device allows it. A filtering `sampler` never samples a depth texture: use a `sampler_comparison` with `textureSampleCompare`, or `textureLoad`.

## Storage Textures

Writeable/readable image surfaces are supported via storage textures (`texture_storage_*`).
Access flags are preserved from WGSL and mapped to the device:

- `read` -> read-only storage access
- `write` -> write-only storage access
- `read_write` -> read+write (when supported)
- `atomic` -> read, write and the texture atomics below, on `r32uint` and `r32sint`

### WGSL example

```wgsl
@group(0) @binding(0) var img: texture_storage_2d<rgba8unorm, write>;
```

Every storage format binds as the format the shader declares. A format or access the device can't bind fails in `Renderer::render` before it takes a frame, and so does a bound texture of another format, one without `STORAGE_BINDING` usage, or one with more than one mip level.

## Texture atomics

An `r32uint` or `r32sint` storage texture with `atomic` access takes texture atomics: `textureAtomicAdd`, `textureAtomicMin`, `textureAtomicMax`, `textureAtomicAnd`, `textureAtomicOr` and `textureAtomicXor`. Each call changes one texel in a single step, however many invocations reach it at once, and returns no value. Exchange and compare-exchange on textures aren't in naga, and `textureLoad` and `textureStore` work on an atomic texture as on any other storage texture.

```wgsl
@group(0) @binding(0) var counts: texture_storage_2d<r32uint, atomic>;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
  textureAtomicAdd(counts, id.xy / 16u, 1u);
}
```

`Shader::new` takes such a shader on every platform, and it runs where the device has `TEXTURE_ATOMIC`: Vulkan, DX12, and Metal from macOS 14 and iOS 17. The browser's WebGPU, WebGL2 and native GL have no texture atomics, so `Renderer::render` refuses the shader there with `RendererError::FeatureError`, whose `feature` names `TEXTURE_ATOMIC`. Bind an `R32Uint` or `R32Sint` texture from [`Renderer::create_storage_texture`](https://fragmentcolor.org/api/core/renderer#create_storage_texture), which takes no extra usage flag for atomics. An `r64uint` storage texture needs `TEXTURE_INT64_ATOMIC`, a feature FragmentColor doesn't request, so `Shader::new` refuses it.

## External textures

WGSL reads video frames through `texture_external`, and FragmentColor fills that slot with an ordinary [Texture](https://fragmentcolor.org/api/texture/texture): `shader.set("video", &texture)`, as for any texture uniform. The shader reads it with `textureSampleBaseClampToEdge`, `textureLoad` and `textureDimensions`.

```wgsl
@group(0) @binding(0) var video: texture_external;
@group(0) @binding(1) var soft: sampler;

@fragment
fn fs_main(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
  let size = vec2<f32>(textureDimensions(video));
  let filtered = textureSampleBaseClampToEdge(video, soft, pixel.xy / size);
  let exact = textureLoad(video, vec2<u32>(pixel.xy));
  return mix(filtered, exact, 0.5);
}
```

Every platform accepts such a shader at `Shader::new`. It draws on devices with `EXTERNAL_TEXTURE`, which wgpu offers on Metal and DX12. On Vulkan, OpenGL, WebGL2 and the browser's WebGPU, `Renderer::render` refuses the shader with `RendererError::FeatureError` before it takes a frame, and the error's `feature` names `EXTERNAL_TEXTURE`.

The slot takes a single-sampled 2D texture in `Rgba8Unorm`, `Bgra8Unorm` or `Rgba16Float` and reads its base mip level, which lets a texture with a mip chain bind too. Raw bytes with a size upload as `Rgba8Unorm`. An 8-bit color image, such as a PNG or a JPEG, decodes into `Rgba8UnormSrgb`, which the slot refuses with `RendererError::BindingError`. Name the format when you create the texture from one: `renderer.create_texture((bytes, TextureFormat::Rgba8Unorm))`.

wgpu counts one external slot as four sampled textures, a sampler and a uniform buffer in each stage that reads it, and `Renderer::render` counts it the same way against the device's limits. A stage that reads four external slots counts 16 sampled textures against `max_sampled_textures_per_shader_stage`.

## Storage Buffers

Structured buffers are supported via `var<storage, read>` or `var<storage, read_write>`
and can contain nested structs/arrays. FragmentColor preserves and applies the WGSL access flags
when creating binding layouts and setting visibility.

### WGSL example

```wgsl
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> ssbo: Buf;
```

Read-only buffers bind with `read-only` storage access; `read_write` allows writes where the device supports it. Each binding is visible to the stages whose entry points use it, so a fragment entry point can write a storage buffer or texture on a device with writable fragment storage. CPU-side updates use the same `shader.set("path", value)` API as uniforms, with array indexing (e.g. `buf.items[2].v`). Buffer byte spans are computed from the WGSL shape (arrays and structs honor stride and alignment automatically) and large buffers are pooled internally.

## Explicit layouts

`@size` and `@align` on the members of a uniform, storage or immediate struct keep their layout on Vulkan, Metal, DX12 and the browser's WebGPU. The WGSL that wgpu writes for the browser drops both attributes, so FragmentColor hands the browser WGSL with a `@size` on each member of such a struct, which pins every offset and the struct's size. The GL backends, WebGL2 and native GL, write a struct into a std140 block member by member and can't keep such a layout, so `Renderer::render` refuses it there with `RendererError::FeatureError`. Spelling the padding out as members works everywhere, as the built-in PBR shader does.

## Immediate data

Immediate data, declared with `var<immediate>`, works on every platform, in render and compute shaders. One root that fits the device's `max_immediate_size` takes native immediates. It becomes a classic uniform buffer instead on the web, on the GL backends, on a device without immediates, and when the shader declares more than one root, with one binding per root in the lowest bind group the shader leaves free. On WebGL2 and native GL the uniform is padded to a multiple of 16 bytes, as those backends take it. A root must therefore also follow the uniform layout rules, such as a 16-byte array stride, and `Shader::new` refuses one that doesn't. An entry point reads at most one root.

A root that isn't a struct takes its bytes whole through `set`, as a storage root does. A struct root also takes each field by name, as in `pc.color`.

## Subgroups and primitive index

`Shader::new` takes subgroup built-ins and functions, such as `@builtin(subgroup_size)` and `subgroupAdd`, and `@builtin(primitive_index)` after `enable primitive_index;`, on every platform. Each runs where the device has its wgpu feature. Elsewhere `Renderer::render` refuses the shader with `RendererError::FeatureError`, whose `feature` names the flag the device lacks.

- Subgroup operations need `SUBGROUP` in compute and fragment shaders and `SUBGROUP_VERTEX` in vertex shaders, and `subgroupBarrier` also needs `SUBGROUP_BARRIER`. Vulkan devices with the full set of subgroup operations offer them, as do Metal GPUs of the Apple7, Mac2 and Metal 3 families and DX12 devices with Shader Model 6.0 wave operations. Only Vulkan offers `SUBGROUP_VERTEX`, and DX12 has no `SUBGROUP_BARRIER`.
- `primitive_index` needs `PRIMITIVE_INDEX`, which DX12, Vulkan devices with geometry shaders, the same Metal GPU families, OpenGL 3.2 and OpenGL ES 3.2 offer.

The browser's WebGPU and WebGL2 offer neither, and native GL offers no subgroups. Write subgroup code in WGSL and leave out `enable subgroups;`, which naga doesn't take yet:

```wgsl
@group(0) @binding(0) var<storage, read_write> sums: array<u32, 64>;

@compute @workgroup_size(64)
fn cs_main(@builtin(local_invocation_index) index: u32) {
  sums[index] = subgroupAdd(1u);
}
```

## Device limits

`Renderer::render` checks each shader against the device before it takes a frame, so every platform returns the same error from the call that caused it, the web included. Each refusal has a typed error with fields:

- `RendererError::LimitError { item, limit, value, max }` for a group past `max_bind_groups`, a binding past `max_bindings_per_bind_group`, more bindings of a kind in one stage than the device takes, a uniform or storage buffer past its binding size limit, a workgroup past the device's sizes, invocations or workgroup memory, and a vertex input at a location past `max_vertex_attributes`. A mesh whose stride or buffers pass the device's limits gets it too.
- `RendererError::FeatureError { item, feature }` for a compute shader on a device without compute, storage a stage of the device can't bind, a storage format or access the device lacks, such as an atomic storage texture on a device without `TEXTURE_ATOMIC`, an external texture on a device without `EXTERNAL_TEXTURE`, an explicit layout on a GL backend, a shader that measures its light roots with `arrayLength` on a device that takes lights through a uniform block, a shader feature the device lacks, such as subgroups, whose `feature` names the wgpu flag, and anything else the device's shader validator refuses.
- `RendererError::BindingError { uniform, expected, found }` for a texture uniform that was never set, or whose texture its slot can't take by format, usage, dimension, sample count or mip levels.
- `RendererError::EntryError { stage }` for a render shader without its `@vertex` or `@fragment` entry point.

JavaScript throws an `Error` named after the class Python raises, such as `LimitError`, with the same fields as properties. Python raises `LimitError`, `FeatureError`, `BindingError` and `EntryError`, each a `FragmentColorError` with the fields as attributes. Swift throws `FragmentColorError.Limit`, `.Feature`, `.Binding` and `.Entry`, and Kotlin the `FragmentColorException` subclasses of the same names.

An unset PBR map slot, such as `base_color_map`, binds a 1x1 default in a render shader, when the slot takes a 2D float texture. One set to a texture `unregister_texture` removed, or one another `Renderer` created, returns `RendererError::TextureNotFoundError`. A `Material` map that this `Renderer` holds no copy of yet binds the PBR default when its slot has one of the five PBR names, sits in a render shader and takes a 2D float texture, as an unset slot does, and returns `RendererError::BindingError` otherwise. That happens on the web before `load`, where the error asks for `load`, and on native for a map that another thread sets while `render` runs, which the next `render` uploads.
