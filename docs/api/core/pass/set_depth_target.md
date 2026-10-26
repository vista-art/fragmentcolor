# Pass::set_depth_target

Attach a depth texture to this pass. Once attached, the renderer builds the pipeline with a matching depth-stencil state and **depth-test is enabled**: fragments behind the current contents of the depth buffer are discarded, and the pass writes to the depth buffer as it draws. That's what you want for 3D meshes that occlude each other.

A Pass has at most one depth attachment. Call `set_depth_target` again to swap it.

The target must be a depth `Texture` (`Depth32Float` is the canonical format) created by the same `Renderer` via [`create_depth_texture`](https://fragmentcolor.org/api/core/renderer#create_depth_texture). A `TextureTarget` always has a color format, so passing one returns an error. A depth texture from another `Renderer`, or one that `unregister_texture` removed, makes `render` return `RendererError::TextureNotFoundError`.

The depth attachment matches the pass's color attachment, the pass's own target or the frame it draws into. A different width or height returns `RendererError::DepthSizeMismatch`, which names both sizes, so a window that resizes needs a depth texture of the new size, or a `resize` of the one it has. A different sample count returns `RendererError::DepthSampleCountMismatch`, so mixing 1× and 4× MSAA in the same pass fails. A shader in the pass that reads the depth texture returns `PassError::TextureConflict`. Read it in a later pass.

A pass that draws a `Model` and has no depth target gets a depth buffer from the renderer. It matches the pass's color attachment in size and sample count and clears at the start of the pass. In a pass that draws models, the shaders that draw none, such as a fullscreen backdrop or an overlay, ignore depth and draw in order, with that buffer or a depth target you set. A pass without models draws without depth unless you call `set_depth_target`, and then every shader in it tests depth.

## Example

A pass that draws 3D meshes with the depth test on, so nearer surfaces hide farther ones.

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
let pass = Pass::from_shader("meshes", &shader);

// Depth test on: closer fragments win, and the pass writes the depth
// buffer, so later draws in the same pass see it.
pass.set_depth_target(&depth)?;

renderer.render(&pass, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
