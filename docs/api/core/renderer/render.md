# Renderer::render(renderable, target)

Renders the given object to the given [Target](https://fragmentcolor.org/api/targets/target).

`renderable` can be a `Shader`, a `Pass`, a `Mesh`, a `Scene`, or any iterable of `Pass` (`Vec<Pass>`, `&[Pass]`, `&[&Pass]`). Passes in an iterable are rendered in order.

A window that is hidden or minimized has no frame to draw on. `render` then returns `RendererError::SurfaceError(SurfaceError::Occluded)`, which Python raises as `OccludedSurfaceError`, a `FragmentColorError`. Skip that frame and draw the next one.

A shader that reads `@location` vertex inputs needs a mesh. Without one, `render` returns `ShaderError::InvalidKey` naming the input. The inputs are the `@location` arguments of the vertex entry point and the `@location` members of its struct arguments, and each one reads the mesh attribute of its own name. The checks on the shader and its textures come first, so an input past `max_vertex_attributes` returns `RendererError::LimitError` instead.

Each mesh a shader draws is checked against the device, and a mesh past one of its limits returns `RendererError::LimitError`, as a shader does: a vertex or instance stride past `max_vertex_buffer_array_stride`, and mesh data that needs a buffer past `max_buffer_size`. Its `item` names the stride or the buffer. WebGL2 takes strides up to 255 bytes. A mesh draws with every `Renderer` that renders it, each from buffers of its own. These checks, and the mesh's own, apply only to the meshes the frame draws: the mesh of a hidden `Model`, or of a `Model` in another pass, isn't checked.

`render(mesh)` draws the mesh with a shader of its own, built from its first vertex. That shader reads `position`, and `color` when the first vertex sets it as four numbers, and it checks the mesh again whenever the mesh changed, so it asks nothing of the mesh's other shaders.

Before it takes a frame, `render` checks each shader against the device and each texture uniform against its slot, so every language returns these errors from the call that caused them, the web included. A group, binding, binding count, buffer size or workgroup past the device's limits, or a vertex input at a location past `max_vertex_attributes`, returns `RendererError::LimitError`, a feature the device lacks `RendererError::FeatureError`, a texture uniform that was never set or whose texture its slot can't take `RendererError::BindingError`, and a render shader without its vertex or fragment entry point `RendererError::EntryError`, each with its fields. An unset PBR map slot binds its 1x1 default instead, when the slot takes a 2D float texture. A texture `unregister_texture` removed, or one from another `Renderer`, returns `RendererError::TextureNotFoundError`, in a PBR map slot too. The [Shader](https://fragmentcolor.org/api/core/shader) page lists the checks and each language's form of these errors.

A `Renderer` draws only into the targets it created. A target from another `Renderer` returns `RendererError::ForeignTarget`, unless every pass is a compute pass, which draws into no target, and a pass target or texture uniform from another `Renderer` returns `RendererError::TextureNotFoundError`. These come before `render` uploads a queued `Material` map or opens a device, so a refused render changes nothing. A `Renderer` that has opened no device yet returns `RendererError::NoContext` for a target without an owner. Each pass is checked against what it draws into before it records: a [depth target](https://fragmentcolor.org/api/core/pass#set_depth_target) of the color attachment's size and sample count, shader outputs and blending the [target](https://fragmentcolor.org/api/core/pass#set_target)'s format takes, a [viewport](https://fragmentcolor.org/api/core/pass#set_viewport) and a [compute dispatch](https://fragmentcolor.org/api/core/pass#set_compute_dispatch) the device takes, and textures that one pass doesn't both write and read. A render pass can't bind a texture it draws into, and no pass can read a texture one of its bindings writes, so a compute dispatch that samples a texture and stores into it returns `PassError::TextureConflict` too.

`render` also returns what the GPU rejects. When a pipeline fails to build or a frame's commands fail, it returns a [DeviceError](https://fragmentcolor.org/api/core/deviceerror), `RendererError::DeviceError` in Rust. Its `kind` tells validation errors, internal errors such as a shader the backend compiler refuses, and running out of memory apart, and its `label` names the step that failed: the render pipeline, the compute pipeline, the light binning pipeline, the frame, or a bind group or texture view inside the frame. When one error causes another, `render` returns the first. In Rust, Python, Swift and Kotlin, a pipeline that failed is never kept, and the next frame builds it again and returns the same error.

A frame that fails is still presented. wgpu clears an image that no valid command drew into, so after a GPU error the window usually shows a cleared frame. On Apple GPUs the renderer submits the commands recorded so far before each compute pass that follows another compute pass. When a pass after that point fails, what the earlier passes drew stays on the image. A shader without its mesh, an instance count past the mesh's instances, a mesh the device's limits or its own checks refuse, a shader or texture the device checks refuse, a target from another `Renderer`, a pass target that is missing, a viewport past the device's 2D limit, and a pass whose own target or compute dispatch fails its checks fail before `render` takes a frame, and the window keeps its last image.

JavaScript gets the GPU's errors late, because WebGPU reports them after `render` has returned. Each error logs a `console.error` when it lands, and the next `render` still throws the first one, as an `Error` with `kind`, `label` and `detail` properties, before doing any work. `load`, `readTexture`, `readStorage`, and `getImage` on a `Texture` or `TextureTarget` wait for the frames before them to report, so they throw a frame's error even when called right after it. A call that waits throws only the error of a frame rendered before it began: the error of a frame rendered while it waits, even one rendered in the same task right after the call, stays for the next call. Only the first error to land throws, and only once; the ones that land after it, before a call takes it, only log. The label names the render pipeline, the compute pipeline, the light binning pipeline or the frame. A pipeline that failed stays in the cache until its error lands, and a frame that draws with it before then reports a frame error. Once the error lands, the pipeline leaves the cache. In a loop that renders each frame, the next `render` throws the error before doing any work, and the one after it builds the pipeline again and reports the same error.

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
