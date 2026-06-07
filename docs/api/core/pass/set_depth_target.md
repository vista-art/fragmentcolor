# Pass::set_depth_target

Attach a depth texture to this pass. Once attached, the renderer builds the pipeline with a matching depth-stencil state and **depth-test is enabled**: fragments behind the current contents of the depth buffer are discarded, and the pass writes to the depth buffer as it draws. That's what you want for 3D meshes that occlude each other.

A Pass has at most one depth attachment. Call `set_depth_target` again to swap it.

The target must be a depth texture (`Depth32Float` is the canonical format) created by the same `Renderer` via [`create_depth_texture`](https://fragmentcolor.org/api/core/renderer/create_depth_texture). The depth attachment's sample count must match the color attachments'. Mixing 1× and 4× MSAA in the same pass returns `RendererError::DepthSampleCountMismatch`.

To **opt out** of depth testing, simply don't call `set_depth_target`. The pass then renders without depth-state, painter's-algorithm style (later draws win).

## Example

A 3D-blob-over-quad render: the painting quad goes first, then 3D blobs with depth-test on so they occlude each other.

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Pass, Shader, Mesh};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;

// One depth attachment shared across the 3D-content pass.
let depth = renderer.create_depth_texture([64, 64]).await?;

let mut mesh = Mesh::new();
mesh.add_vertex([0.0, 0.0, 0.0]);
mesh.add_vertex([1.0, 0.0, 0.0]);
mesh.add_vertex([0.0, 1.0, 0.0]);
mesh.add_vertex([1.0, 1.0, 0.0]);
let shader = Shader::from_mesh(&mesh);
let pass = Pass::from_shader("blobs", &shader);

// Depth-test on — closer fragments win, the pass writes to the depth
// buffer so subsequent draws within the same pass see the depth.
pass.set_depth_target(&depth)?;

renderer.render(&pass, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
