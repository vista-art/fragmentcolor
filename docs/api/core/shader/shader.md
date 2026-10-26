# Shader

The [Shader](https://fragmentcolor.org/api/core/shader) object is the main building block in [FragmentColor](https://fragmentcolor.org).

It takes a WGSL or GLSL shader source as input, parses it, validates it, and exposes the uniforms as keys.

`Shader::new` refuses a source that no device FragmentColor opens can run, with a `ShaderError` that names the problem: a feature outside the set FragmentColor requests (`f16`, subgroups, `texture_external`, texture atomics), two globals on one `@group` and `@binding`, writable storage in a `@vertex` entry point, an immediate root that breaks the uniform layout rules, and two vertex inputs with one name, which would read one mesh attribute.

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

Every storage format binds as the format the shader declares. A format or access the device can't bind fails in `Renderer::render` before any GPU work, and so does a bound texture of another format, one without `STORAGE_BINDING` usage, or one with more than one mip level.

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

`@size` and `@align` on the members of a uniform, storage or immediate struct keep their layout on every platform. The WGSL that wgpu writes for the browser's WebGPU drops both attributes, so FragmentColor hands the browser WGSL with a `@size` on each member of such a struct, which pins every offset and the struct's size.

## Immediate data

Immediate data, declared with `var<immediate>`, works on every platform, in render and compute shaders. One root that fits the device's `max_immediate_size` takes native immediates. It becomes a classic uniform buffer instead on the web, on a device without immediates, and when the shader declares more than one root, with one binding per root in the lowest bind group the shader leaves free. A root must therefore also follow the uniform layout rules, such as a 16-byte array stride, and `Shader::new` refuses one that doesn't. An entry point reads at most one root.

A root that isn't a struct takes its bytes whole through `set`, as a storage root does. A struct root also takes each field by name, as in `pc.color`.

## Device limits

`Renderer::render` checks each shader against the device before it takes a frame, so every platform returns the same error from the call that caused it, the web included. It refuses a group past `max_bind_groups`, a binding past `max_bindings_per_bind_group`, more bindings of a kind in one stage than the device takes, a uniform or storage buffer past its binding size limit, a render shader without its `@vertex` or `@fragment` entry point, a compute shader on a device without compute or with a workgroup past the limits, and anything else the device's shader validator refuses. A texture uniform with no texture, or with a texture its slot can't take by format, usage, dimension or sample count, fails the same way.
