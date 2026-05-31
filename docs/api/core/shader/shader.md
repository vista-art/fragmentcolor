# Shader

The [Shader](https://fragmentcolor.org/api/core/shader) object is the main building block in [FragmentColor](https://fragmentcolor.org).

It takes a WGSL or GLSL shader source as input, parses it, validates it, and exposes the uniforms as keys.

`Shader::new` accepts a source string, a registry slug like `"sdf2d/circle"`, an `https://` URL, a local file path, or an array mixing any of those. Array parts are deduplicated by hash and concatenated in order, so you can pull pure helper functions from the public registry at `https://fragmentcolor.org/shaders/` into your own shader without copy-pasting. Override the registry base with [Shader::set_registry](https://fragmentcolor.org/api/core/shader#shaderset_registry).

To draw your shader, you must use your [Shader](https://fragmentcolor.org/api/core/shader) instance as input to a [Renderer](https://fragmentcolor.org/api/core/renderer).

You can compose [Shader](https://fragmentcolor.org/api/core/shader) instances into a [Pass](https://fragmentcolor.org/api/core/pass) object to create more complex rendering pipelines.

You can also create renderings with multiple Render Passes by passing an array of [Pass](https://fragmentcolor.org/api/core/pass) instances to [Renderer::render](https://fragmentcolor.org/api/core/renderer#renderrender).

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

2D, 3D, cube, and array variants are all supported; the correct view dimension is inferred from the WGSL declaration. Integer textures map to `Sint` / `Uint` sample types; float textures use filterable float when the device allows it.

## Storage Textures

Writeable/readable image surfaces are supported via storage textures (`texture_storage_*`).
Access flags are preserved from WGSL and mapped to the device:

- `read` -> read-only storage access
- `write` -> write-only storage access
- `read_write` -> read+write (when supported)

### WGSL example

```wgsl
@group(0) @binding(0) var img: texture_storage_2d<rgba8unorm, write>;
```

The declared storage format flows through to the binding layout untouched. You're responsible for picking a format and access mode the adapter supports.

## Storage Buffers

Structured buffers are supported via `var<storage, read>` or `var<storage, read_write>`
and can contain nested structs/arrays. FragmentColor preserves and applies the WGSL access flags
when creating binding layouts and setting visibility.

### WGSL example

```wgsl
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> ssbo: Buf;
```

Read-only buffers bind with `read-only` storage access; `read_write` allows writes where the device supports it. CPU-side updates use the same `shader.set("path", value)` API as uniforms, with array indexing (e.g. `buf.items[2].v`). Buffer byte spans are computed from the WGSL shape (arrays and structs honor stride and alignment automatically) and large buffers are pooled internally.

## Push Constants

Push constants are supported with `var<push_constants>` in all platforms.

They will fallback to regular uniform buffers when:

- push_constants are not natively supported (ex. on Web),
- multiple push-constant roots are declared, or
- the total push-constant size exceeds the device limit.

In fallback mode, push constants are rewritten as classic uniform buffers in a newly allocated bind group (one binding per push-constant root). The fallback group goes into the next free slot (`max_existing_group + 1`); FragmentColor does not check the device's `max_bind_groups` limit, so a shader that already uses many bind groups *and* push constants can exceed it. Render pipelines fall back today; compute pipeline fallback may follow.
