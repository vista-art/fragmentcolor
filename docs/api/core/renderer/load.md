# Renderer::load

Realize every pending GPU upload referenced by a renderable. Material's
texture setters (`base_color_texture`, `metallic_roughness_texture`, …)
accept paths, bytes, and URLs in addition to
already-uploaded [`Texture`](https://fragmentcolor.org/api/texture/texture)
handles; the path / bytes / URL variants are stored on the Material's
Shader as *pending* uploads. `load` walks every Shader the renderable
visits, drains its pending list, calls
[`Renderer::create_texture`](https://fragmentcolor.org/api/core/renderer#create_texture)
for each, and writes the resulting `Texture` into the matching uniform.

An upload that fails stays pending and the others still bind, so a good
normal map binds even when the base color path is missing. `load` returns
the first failure, and so does every later `load`, and every native
`render`, until the input is fixed or replaced. A map the shader doesn't
declare fails the `load` that uploads it, once, and the other maps still
bind. The latest setter call for a map wins, even one made while `load` is
uploading the older input, which is then dropped.

On native targets, [`Renderer::render`](https://fragmentcolor.org/api/core/renderer)
runs `load` first whenever the renderable has pending uploads, so an
explicit `load` is optional there. Reach for it when you want to pay the
decode and upload cost outside the render loop, for example in a loading
screen that prepares the scene before the first frame goes out.

On the web, `render` can't wait for an upload, so it leaves pending uploads
alone. Await `load` before the first `render`. Until then, a PBR Material
draws with its 1×1 default maps.

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
mesh.add_vertex(Vertex::pbr([0.0, 0.5, 0.0]).set(Vertex::UV0, [0.5, 1.0]));

// The material keeps the encoded image as a pending upload.
let material = Material::pbr().base_color_texture(&image);
let model = Model::new(mesh, material);
let scene = Scene::new();
scene.add(&model)?;

// Decode and upload it now, so the first render only draws.
renderer.load(&scene).await?;
renderer.render(&scene, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
