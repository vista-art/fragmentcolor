# Renderer::load

Uploads into this `Renderer` every Material map of a renderable that it
lacks, or holds from an older set. Material's texture setters
(`base_color_texture`, `metallic_roughness_texture`, …) take an
already-uploaded [`Texture`](https://fragmentcolor.org/api/texture/texture)
or a source: bytes, a path, a URL or a `Mipmap`. The Material keeps a source
as a map with one texture id. `load` walks every Shader the renderable visits
and decodes each map this `Renderer` needs into a copy of its own, under that id.

An upload that fails keeps no copy, and the others still upload: a good
normal map draws even when the base color path is missing. `load` returns
the first failure, as does every later `load` and every native `render`,
until the source is fixed or replaced. The latest setter call for a map
wins, even one made while `load` is uploading the older source, which is
then dropped. A slot the shader doesn't declare never becomes a map, since
the setter logs a warning and leaves the Material as it was.

Each `Renderer` holds copies of its own. One Material draws with every
`Renderer` that loads it, and a copy goes when its `Renderer` or the
Material drops. A map set again draws its old copy until this `Renderer`
loads the new one.

On native targets, [`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render)
runs `load` first, which makes an explicit `load` optional there. Reach for
it when you want to pay the decode and upload cost outside the render loop,
for example in a loading screen that prepares the scene before the first
frame goes out.

On the web, `render` can't wait for an upload and uploads nothing. Await
`load` in each `Renderer` before its first `render`. Until then, a PBR
Material draws with its 1×1 default maps, and a custom shader's map slot
fails the render with a `BindingError` that asks for `load`.

In JavaScript, `load` first waits for the frames rendered before it to
report their GPU errors, and throws the first one, as
[`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render) describes.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let image = std::fs::read("examples/assets/logo.png")?;
# /*[js]
# const image = new Uint8Array(await (await fetch("/healthcheck/public/favicon.png")).arrayBuffer());
# */
# /*[python]
# image = open("examples/assets/logo.png", "rb").read()
# */
# /*[swift]
# import Foundation
# let image = try Data(contentsOf: URL(fileURLWithPath: "examples/assets/logo.png"))
# */
# /*[kotlin]
# val image = java.io.File("examples/assets/logo.png").readBytes()
# */
use fragmentcolor::{Material, Mesh, Model, Renderer, Scene, Vertex};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

let mesh = Mesh::new();
mesh.add_vertices([
    Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]),
    Vertex::pbr([-0.5, -0.5, 0.0]).set(Vertex::UV0, [0.0, 0.0]),
    Vertex::pbr([0.5, -0.5, 0.0]).set(Vertex::UV0, [1.0, 0.0]),
]);

// The material keeps the encoded image as a map.
let material = Material::pbr().base_color_texture(&image);
let model = Model::new(mesh, material);
let scene = Scene::new();
scene.add(&model)?;

// Decode and upload it now, so the first render only draws.
renderer.load(&scene).await?;
renderer.render(&scene, &target)?;

// A second Renderer draws the same map from a copy of its own.
let other = Renderer::new();
let frame = other.create_texture_target([64, 64]).await?;
other.load(&scene).await?;
other.render(&scene, &frame)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
