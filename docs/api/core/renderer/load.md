# Renderer::load

Realize every pending GPU upload referenced by a renderable. Material's
texture setters (`base_color_texture`, `metallic_roughness_texture`, …)
accept paths, bytes, URLs, and `DynamicImage` values in addition to
already-uploaded [`Texture`](https://fragmentcolor.org/api/texture/texture)
handles; the path / bytes / URL variants are stored on the Material's
Shader as *pending* uploads. `load` walks every Shader the renderable
visits, drains its pending list, calls
[`Renderer::create_texture`](https://fragmentcolor.org/api/core/renderer/create_texture)
for each, and writes the resulting `Texture` into the matching uniform.

[`Renderer::render`](https://fragmentcolor.org/api/core/renderer/render)
calls `load` automatically the first time it sees a renderable with
pending uploads, so explicit `load` is optional. Reach for it when you
want to amortize the decode + upload cost outside the render loop — for
example, in a loading screen that prepares the scene before the first
frame goes out.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Material, Mesh, Model, Renderer, Scene, Vertex};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64u32, 64u32]).await?;

let mesh = Mesh::new();
mesh.add_vertex(
    Vertex::new([0.0, 0.5, 0.0])
        .set(Vertex::NORMAL, [0.0, 0.0, 1.0])
        .set(Vertex::UV0, [0.5, 1.0])
        .set(Vertex::COLOR0, [1.0, 1.0, 1.0, 1.0])
        .set(Vertex::UV1, [0.0, 0.0]),
);
// Raw 2×2 RGBA pixel bytes — uploaded lazily by `Renderer::load` below.
// In practice the loader hands the setter encoded PNG/JPEG bytes (from a
// BIN chunk) or a file path (from a URI); the same `Into<TextureInput>`
// vocabulary covers all of them.
let red_pixels: Vec<u8> = vec![
    255,   0,   0, 255,    0, 255,   0, 255,
      0,   0, 255, 255,  255, 255, 255, 255,
];
let material = Material::pbr()?.base_color_texture((red_pixels, [2u32, 2u32]));
let model = Model::new(mesh, material);
let scene = Scene::new();
scene.add(&model)?;

// Eager prewarm — uploads the pending texture(s) so the next render is
// GPU-only.
renderer.load(&scene).await?;
renderer.render(&scene, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
