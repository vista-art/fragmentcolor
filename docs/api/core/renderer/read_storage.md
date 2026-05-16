# Renderer::read_storage

Round-trip the GPU-side bytes of a [`Shader`](https://fragmentcolor.org/api/shaders/shader)'s
storage binding back to the CPU. Submits any pending uploads, copies the
storage buffer into a pooled readback staging buffer, maps it, and returns
the bytes.

This is the only way to observe what a compute pass *wrote* into a storage
binding. The cheap CPU-mirror accessors —
[`Shader::get`](https://fragmentcolor.org/api/shaders/shader/get) and
[`Shader::get_bytes`](https://fragmentcolor.org/api/shaders/shader/get_bytes)
— reflect what was last set on the CPU side; GPU writes are not visible
through them.

`read_storage` is `async` on every platform because the underlying
`map_async` requires an awaited callback. On native the implementation
drives the device forward (`device.poll(Wait)`) before awaiting, so callers
don't have to spin their own poll loop. On Web the browser schedules the
callback; the helper just awaits.

Returns
[`RendererError::StorageBindingNotFound`](https://fragmentcolor.org/api/core/renderer/error)
when the shader does not declare a storage binding called `binding`, or
when no render pass has yet materialised the GPU-side buffer (storage
buffers are allocated lazily on first bind).

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use bytemuck;
use fragmentcolor::{Pass, Renderer, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([16u32, 16u32]).await?;

let compute = Shader::new(
    r#"
    struct Out { values: array<f32, 4> };
    @group(0) @binding(0) var<storage, read_write> out: Out;
    @compute @workgroup_size(1) fn main() {
        out.values[0] = 1.0;
        out.values[1] = 2.0;
        out.values[2] = 3.0;
        out.values[3] = 4.0;
    }
    "#,
)?;

let pass = Pass::compute("seed");
pass.set_compute_dispatch(1, 1, 1);
pass.add_shader(&compute);
renderer.render(&pass, &target)?;

let bytes = renderer.read_storage(&compute, "out").await?;
let values: &[f32] = bytemuck::cast_slice(&bytes);
# assert_eq!(values, &[1.0, 2.0, 3.0, 4.0]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
