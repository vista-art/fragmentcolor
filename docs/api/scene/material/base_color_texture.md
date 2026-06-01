# Material::base_color_texture

Bind a texture to the `base_color_map` slot. The default PBR shader samples it in `fs_main` and multiplies by the factor. The albedo for each fragment is `material.base_color * textureSample(base_color_map, sampler, in.uv)`.

Accepts any `Into<TextureInput>`:

- A pre-built [`&Texture`](https://fragmentcolor.org/api/texture/texture). The setter stores the texture's ID immediately and the GPU texture stays Arc-shared across every Material that points at it. Passing the same `&shared_texture` to N Materials produces one GPU upload and N shader-uniform references: the cheap way to reuse one texture map across a scene full of objects without paying for N texture allocations.
- Bytes / path / URL / `DynamicImage`. The setter queues the input on the Material's Shader, and the renderer drains queued uploads on the first [`Renderer::render`](https://fragmentcolor.org/api/core/renderer/render) (or earlier via the explicit [`Renderer::load`](https://fragmentcolor.org/api/core/renderer/load)).

Unset, this slot resolves to a 1×1 white default the renderer hands out
lazily, so calling `Material::pbr()?` without binding a
texture renders correctly under the factor alone.

## Errors are surfaced lazily

The setter itself is infallible: it queues the upload (lazy path) or
takes an Arc-clone (eager path) and returns. Failures from the lazy
path (file not found, decode error, unsupported format) surface when
the renderer actually drains the queue: either at first render or when
you call [`Renderer::load(&material).await`](https://fragmentcolor.org/api/core/renderer/load).
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
let albedo = renderer.create_texture(&[
    255, 200, 120, 255,
    255,  240, 180, 255,
    230,  180, 100, 255,
    255,  220, 150, 255,
][..]).await?;

// 279 blob Materials all sample the same uploaded `albedo` — one GPU
// texture, 279 shader references.
let mut blob_materials = Vec::with_capacity(279);
for _ in 0..279 {
    blob_materials.push(Material::pbr()?.base_color_texture(&albedo));
}
# let _blob_materials = blob_materials;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
