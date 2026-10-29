# Material::base_color_texture

Bind a texture to the `base_color_map` slot. The default PBR shader samples it in `fs_main` and multiplies by the factor. The albedo for each fragment is `material.base_color * textureSample(base_color_map, pbr_sampler, uv) * vertex_color`, where `uv` is `slot_uv(material.uv[0], in.uv0, in.uv1)`: the UV set [`texture_uv`](https://fragmentcolor.org/api/scene/material#texture_uv) picks for the slot, through the slot's [`texture_transform`](https://fragmentcolor.org/api/scene/material#texture_transform).

Accepts any `Into<TextureInput>`, the inputs [`Renderer::create_texture`](https://fragmentcolor.org/api/core/renderer#create_texture) takes:

- A pre-built [`&Texture`](https://fragmentcolor.org/api/texture/texture). The setter stores the texture's ID immediately and the GPU texture stays Arc-shared across every Material that points at it. Passing the same `&shared_texture` to N Materials produces one GPU upload and N shader-uniform references: the cheap way to reuse one texture map across a scene full of objects without paying for N texture allocations.
- Encoded bytes, raw pixels with their size as `(bytes, [w, h])`, bytes with a format as `(bytes, format)`, bytes with every option as `(bytes, options)`, a path, a URL, or a [`Mipmap`](https://fragmentcolor.org/api/texture/mipmap). The setter queues the input on the Material's Shader. [`Renderer::load`](https://fragmentcolor.org/api/core/renderer) uploads it, and on native targets the first [`Renderer::render`](https://fragmentcolor.org/api/core/renderer) runs `load` for you.

Each language spells these inputs the way its `create_texture` does, and the other four texture setters take the same ones:

- **JavaScript:** `baseColorTexture(source, options?)`, where `options` is a size such as `[w, h]`, a `TextureFormat`, or `{ size?, format?, mipmaps?, sampler?, usage? }`.
- **Python:** `base_color_texture(texture, size=None, format=None, mipmaps=None, sampler=None, usage=None)`. A NumPy array brings its own size, and a `size` that differs from it raises `InvalidInputError`.
- **Swift and Kotlin:** `baseColorTexture(bytes)`, `baseColorTexture(bytes, format)`, `baseColorTexture(bytes, [w, h])`, `baseColorTexture(bytes, size)`, `baseColorTexture(bytes, options)`, `baseColorTexture(path)`, `baseColorTexture(path, options)` and `baseColorTexture(chain)`. Bytes are `Data` or `[UInt8]` in Swift and `ByteArray` in Kotlin, and raw pixels with a `[w, h]` also come one value per byte, as `[Int]` or `List<Int>`. The bytes forms without `options` take an optional `usage`, and a `[w, h]` that isn't 2 or 3 numbers throws.

Unset, this slot resolves to a 1×1 white default the renderer hands out
lazily, so calling `Material::pbr()` without binding a
texture renders correctly under the factor alone.

## Errors are surfaced lazily

The setter itself is infallible: it queues the upload (lazy path) or
takes an Arc-clone (eager path) and returns. Failures from the lazy
path (file not found, decode error, unsupported format) surface when
the queue drains: when you call
[`Renderer::load(&material).await`](https://fragmentcolor.org/api/core/renderer),
or at the first render on native targets.
Until then the Material renders against its 1×1 default and a
`log::warn!` line names the slot.

For deterministic error handling, pre-build the texture eagerly:
`renderer.create_texture(path).await?` returns a `Texture` that can't
fail at setter time, and `Material::base_color_texture(&texture)` takes
an Arc-shared reference. The lazy path stays useful when the URL /
path resolves at render time but the caller doesn't want to await yet.

## Example: sharing one texture across many Materials

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let albedo_bytes: Vec<u8> = vec![
    255, 200, 120, 255,
    255, 240, 180, 255,
    230, 180, 100, 255,
    255, 220, 150, 255,
];
let albedo = renderer.create_texture((albedo_bytes, [2, 2])).await?;

// Every Material that points at `albedo` reuses the same uploaded GPU
// texture; passing the same handle into N Material instances costs one
// upload and N shader-uniform references.
let material = Material::pbr().base_color_texture(&albedo);

// Raw pixels with their size upload when the scene loads.
let tint: Vec<u8> = vec![255, 128, 0, 255];
let tinted = Material::pbr().base_color_texture((tint, [1, 1]));
# let _ = (material, tinted);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
