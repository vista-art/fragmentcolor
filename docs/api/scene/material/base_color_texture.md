# Material::base_color_texture

Bind a texture to the canonical `base_color_map` slot. The default PBR
shader samples it in `fs_main` and multiplies by the factor: per-fragment
albedo is `material.base_color * textureSample(base_color_map, sampler, in.uv)`.

Accepts any `Into<TextureInput>`:

- A pre-built [`&Texture`](https://fragmentcolor.org/api/texture/texture) — the eager path. The setter stores the texture's ID immediately and the GPU texture stays Arc-shared across every Material that points at it. Passing the same `&shared_texture` to N Materials produces one GPU upload + N shader-uniform references — the cheap way to wallpaper a brush stroke across a scene full of impasto blobs without paying for N texture allocations.
- Bytes / path / URL / `DynamicImage` — the lazy path. The setter queues the input on the Material's Shader; the renderer drains queued uploads on the first [`Renderer::render`](https://fragmentcolor.org/api/core/renderer/render) (or earlier via the explicit [`Renderer::load`](https://fragmentcolor.org/api/core/renderer/load)).

Unset, this slot resolves to a 1×1 white default the renderer hands out
lazily — so calling `Material::pbr()?` without binding a
texture renders correctly under the factor alone.

## Example — sharing one texture across many Materials

```rust,no_run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Renderer};

let renderer = Renderer::new();
let albedo = renderer.create_texture(&[
    255u8, 200, 120, 255,
    255,  240, 180, 255,
    230,  180, 100, 255,
    255,  220, 150, 255,
][..]).await?;

// 279 blob Materials all sample the same uploaded `albedo` — one GPU
// texture, 279 shader references.
let blob_materials: Vec<_> = (0..279)
    .map(|_| Material::pbr().unwrap().base_color_texture(&albedo))
    .collect();
# let _ = blob_materials;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
