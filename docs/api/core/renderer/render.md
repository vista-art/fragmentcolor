# Renderer::render(renderable, target)

Renders the given object to the given [Target](https://fragmentcolor.org/api/targets/target).

`renderable` can be a `Shader`, a `Pass`, a `Mesh`, a `Scene`, or any iterable of `Pass` (`Vec<Pass>`, `&[Pass]`, `&[&Pass]`). Passes in an iterable are rendered in order.

A window that is hidden or minimized has no frame to draw on. `render` then returns `RendererError::SurfaceError(SurfaceError::Occluded)`, which Python raises as `OccludedSurfaceError`, a `FragmentColorError`. Skip that frame and draw the next one.

A shader that reads `@location` vertex inputs needs a mesh. Without one, `render` returns `ShaderError::InvalidKey` naming the input. Meshes feed only `@location` arguments of the vertex entry point, so when a shader takes its inputs as members of a struct, the error names the member instead.

`render` also returns what the GPU rejects. When a pipeline fails to build or a frame's commands fail, it returns `RendererError::ValidationError`, labeled with the step that failed: the render pipeline, the compute pipeline, the frame, or a bind group or texture view inside the frame. That covers validation errors, internal errors such as a shader the backend compiler refuses, and running out of memory. When one error causes another, `render` returns the first. A pipeline that failed is never kept, and the next frame builds it again and returns the same error.

A frame that fails is still presented. wgpu clears an image that no valid command drew into, so after a GPU error the window shows a cleared frame. A shader without its mesh and an instance count past the mesh's instances fail before `render` takes a frame, and the window keeps its last image.

JavaScript gets the GPU's errors late, because WebGPU reports them after `render` has returned. Each error logs a `console.error` when it lands, and the next `render` throws the first one as an `Error` before doing any work. `load`, `readTexture`, `readStorage`, and `getImage` on a `Texture` or `TextureTarget` wait for the frames before them to report, so they throw a frame's error even when called right after it. An error throws once; the ones that land after it, before a call takes it, only log. The label names the render pipeline, the compute pipeline or the frame. A pipeline that failed leaves the cache when its error lands, so the next frame that draws with it builds it again and reports the same error.

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
