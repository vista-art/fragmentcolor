# Shader

The [Shader](https://fragmentcolor.org/api/core/shader) object is the main building block in [FragmentColor](https://fragmentcolor.org).

It takes a WGSL or GLSL shader source as input, parses it, validates it, and exposes the uniforms as keys.

To draw your shader, you must use your [Shader](https://fragmentcolor.org/api/core/shader) instance as input to a [Renderer](https://fragmentcolor.org/api/core/renderer).

You can compose [Shader](https://fragmentcolor.org/api/core/shader) instances into a [Pass](https://fragmentcolor.org/api/core/pass) object to create more complex rendering pipelines.

You can also create renderings with multiple Render Passes by using multiple [Pass](https://fragmentcolor.org/api/core/pass) instances to a [Frame](https://fragmentcolor.org/api/core/frame) object.

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

### Notes

- Binding sizes are aligned to 16 bytes for layout correctness; this is handled automatically.
- Large uniform blobs are uploaded via an internal buffer pool.

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

### Notes

- 2D/3D/Cube and array variants are supported; the correct view dimension is inferred.
- Integer textures map to Sint/Uint sample types; float textures use filterable float when possible.

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

### Notes

- The declared storage format is respected when creating the binding layout.
- User must ensure the adapter supports the chosen format/access mode.

## Storage Buffers

Structured buffers are supported via `var<storage, read>` or `var<storage, read_write>`
and can contain nested structs/arrays. FragmentColor preserves and applies the WGSL access flags
when creating binding layouts and setting visibility.

### WGSL example

```wgsl
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> ssbo: Buf;
```

### Notes

- Read-only buffers are bound with `read-only` storage access; `read_write` allows writes when supported.
- Buffer byte spans are computed from WGSL shapes; arrays/structs honor stride and alignment.
- CPU-side updates use the same set("path", value) and get_bytes("path") APIs as uniforms, with array indexing supported (e.g., `buf.items[2].v`).
- Large buffers are uploaded via a dedicated storage buffer pool.

## Push Constants

Push constants are supported with `var<push_constants>` in all platforms.

They will fallback to regular uniform buffers when:

- push_constants are not natively supported (ex. on Web),
- multiple push-constant roots are declared, or
- the total push-constant size exceeds the device limit.

### Notes

In fallback mode, FragmentColor rewrites push constants into classic uniform buffers
placed in a newly allocated bind group. In this case:

- A bind group slot will be used by this fallback group (allocated as max existing group + 1).
- There is no check for the max bind groups supported.
- If you use push constants and many bind groups, very high group indices can exceed device limits.
- Each push-constant root becomes one uniform buffer binding in the fallback group.
- Currently, the fallback is applied for render pipelines; compute pipeline fallback may be added later.
