# Renderer::render(renderable, target)

Renders the given object to the given [Target](https://fragmentcolor.org/api/targets/target).

`renderable` can be a `Shader`, a `Pass`, a `Mesh`, a `Scene`, or any iterable of `Pass` (`Vec<Pass>`, `&[Pass]`, `&[&Pass]`). Passes in an iterable are rendered in order.

A window that is hidden or minimized has no frame to draw on. `render` then returns `RendererError::SurfaceError(SurfaceError::Occluded)`, which Python raises as `OccludedSurfaceError`, a `FragmentColorError`. Skip that frame and draw the next one.

A shader that reads `@location` vertex inputs needs a mesh. Without one, `render` returns `ShaderError::InvalidKey` naming the input. The inputs are the `@location` arguments of the vertex entry point and the `@location` members of its struct arguments, and each one reads the mesh attribute of its own name.

`render` also returns what the GPU rejects. When a pipeline fails to build or a frame's commands fail, it returns a [DeviceError](https://fragmentcolor.org/api/core/deviceerror), `RendererError::DeviceError` in Rust. Its `kind` tells validation errors, internal errors such as a shader the backend compiler refuses, and running out of memory apart, and its `label` names the step that failed: the render pipeline, the compute pipeline, the light binning pipeline, the frame, or a bind group or texture view inside the frame. When one error causes another, `render` returns the first. In Rust, Python, Swift and Kotlin, a pipeline that failed is never kept, and the next frame builds it again and returns the same error.

A frame that fails is still presented. wgpu clears an image that no valid command drew into, so after a GPU error the window usually shows a cleared frame. On Apple GPUs the renderer submits the commands recorded so far before each compute pass that follows another compute pass. When a pass after that point fails, what the earlier passes drew stays on the image. A shader without its mesh and an instance count past the mesh's instances fail before `render` takes a frame, and the window keeps its last image.

JavaScript gets the GPU's errors late, because WebGPU reports them after `render` has returned. Each error logs a `console.error` when it lands, and the next `render` still throws the first one, as an `Error` with `kind`, `label` and `detail` properties, before doing any work. `load`, `readTexture`, `readStorage`, and `getImage` on a `Texture` or `TextureTarget` wait for the frames before them to report, so they throw a frame's error even when called right after it. A call that waits throws only the error of a frame rendered before it began: the error of a frame rendered while it waits stays for the next call. Only the first error to land throws, and only once; the ones that land after it, before a call takes it, only log. The label names the render pipeline, the compute pipeline, the light binning pipeline or the frame. A pipeline that failed stays in the cache until its error lands, and a frame that draws with it before then reports a frame error. Once the error lands, the pipeline leaves the cache. In a loop that renders each frame, the next `render` throws the error before doing any work, and the one after it builds the pipeline again and reports the same error.

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
