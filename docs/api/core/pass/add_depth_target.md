# Pass::add_depth_target

Select a depth attachment for this pass.

The target must be a stable texture created by the same Renderer with create_depth_texture().

When a depth target is attached, the renderer will create a render pipeline with a depth-stencil
matching the texture format (e.g., `Depth32Float`) of the created texture.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Pass, Shader, Mesh};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

// Create a depth texture usable as a per-pass attachment
let depth = renderer.create_depth_texture([64, 64]).await?;

let mut mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0, 0.0]);
mesh.add_vertex([1.0, 0.0, 0.0]);
mesh.add_vertex([0.0, 1.0, 0.0]);
mesh.add_vertex([1.0, 1.0, 0.0]);
let shader = Shader::from_mesh(&mesh);
let pass = Pass::from_shader("scene", &shader);

// Attach depth texture to enable depth testing.
// Pipeline will include a matching depth-stencil state
pass.add_depth_target(&depth)?;

// Render as usual
renderer.render(&pass, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
