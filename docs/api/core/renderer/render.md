# Renderer::render(renderable, target)

Renders the given object to the given [Target](https://fragmentcolor.org/api/targets/target).

`renderable` can be a `Shader`, a `Pass`, a `Mesh`, a `Scene`, or any iterable of `Pass` (`Vec<Pass>`, `&[Pass]`, `&[&Pass]`). Passes in an iterable are rendered in order.

A window that is hidden or minimized has no frame to draw on. `render` then returns `RendererError::SurfaceError(SurfaceError::Occluded)`, which Python raises as `OccludedSurfaceError`, a `FragmentColorError`. Skip that frame and draw the next one.

A shader that reads `@location` vertex inputs needs a mesh. Without one, `render` returns `ShaderError::InvalidKey` naming the input.

`render` also returns what the GPU rejects. When a pipeline fails to build or a frame's commands fail validation, it returns `RendererError::ValidationError`, labeled with the render pipeline, the compute pipeline or the frame, and presents nothing from that frame. A pipeline that failed is never kept, and the next frame returns the same error. On the web these errors go to the browser console instead: WebGPU reports them asynchronously, and `render` is synchronous.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([10, 10]).await?;
let shader = Shader::default();

renderer.render(&shader, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
