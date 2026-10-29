# Material::base_color_texture

Bind a texture to the `base_color_map` slot. The default PBR shader samples it in `fs_main` and multiplies by the factor. The albedo for each fragment is `material.base_color * textureSample(base_color_map, pbr_sampler, uv) * vertex_color`, where `uv` is `slot_uv(material.uv[0], in.uv0, in.uv1)`: the UV set [`texture_uv`](https://fragmentcolor.org/api/scene/material#texture_uv) picks for the slot, through the slot's [`texture_transform`](https://fragmentcolor.org/api/scene/material#texture_transform).

Accepts any `Into<TextureInput>`:

- A pre-built [`&Texture`](https://fragmentcolor.org/api/texture/texture). The setter stores the texture's ID immediately and the GPU texture stays Arc-shared across every Material that points at it. Passing the same `&shared_texture` to N Materials produces one GPU upload and N shader-uniform references: the cheap way to reuse one texture map across a scene full of objects without paying for N texture allocations. The texture belongs to the `Renderer` that created it, and another `Renderer` refuses to draw it.
- Bytes, a path, a URL or a `Mipmap`. The Material keeps this source as a map. Each `Renderer` that draws the Material uploads a copy of its own at [`Renderer::load`](https://fragmentcolor.org/api/core/renderer#load), or on native targets at its first [`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render), and frees it when the `Renderer` or the Material drops. A map set again draws its old copy until the `Renderer` loads the new one.

To share a map across Renderers, pass bytes, a path, a URL or a `Mipmap`.

Unset, this slot resolves to a 1×1 white default the renderer hands out
lazily, so calling `Material::pbr()` without binding a
texture renders correctly under the factor alone.

## Memory

A map holds its source for as long as the slot keeps it. That is the file's
bytes for an encoded PNG or JPEG, the pixels for raw bytes, a NumPy array or
a glTF image, the container for KTX2, and the chain for a `Mipmap`. A path or
a URL holds only its string, and each `Renderer` reads or fetches it again.
`renderer.create_texture` keeps nothing on the CPU, and the texture it returns
draws in that one `Renderer`.

## Errors

The setter checks the slot at once. A slot the shader doesn't declare as a
texture, or an empty input, logs a warning and leaves the Material as it was.

A source that fails to upload (file not found, decode error, unsupported
format) fails [`Renderer::load(&material).await`](https://fragmentcolor.org/api/core/renderer#load),
or the native render that runs it. That `Renderer` keeps no copy and tries
again at each later call, until the source is fixed or replaced, while the
other maps upload.

On the web, `render` uploads nothing, so await `load` in each `Renderer`
before its first frame. Until then a built-in PBR slot draws its 1×1 default,
and a custom shader's slot fails the render with a `BindingError` that asks
for `load`.

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
# let _ = material;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
