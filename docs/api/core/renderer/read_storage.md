# Renderer::read_storage

Round-trip the GPU-side bytes of a [`Shader`](https://fragmentcolor.org/api/core/shader)'s
storage binding back to the CPU. Submits any pending uploads, copies the
storage buffer into a pooled readback staging buffer, maps it, and returns
the bytes.

This is the only way to observe what a compute pass *wrote* into a storage
binding. [`Shader::get`](https://fragmentcolor.org/api/core/shader) returns
what was last set on the CPU side, so GPU writes don't show up through it.
A [`Shader::set`](https://fragmentcolor.org/api/core/shader#set) on a field
or an item of the root uploads only the 4-byte words that hold it at the next
render that binds it, so the rest of the root keeps what a compute pass wrote
there. Setting one `f16` also writes the `f16` that shares its word, from the
shader's copy. When two Renderers draw one shader, each uploads only the sets
it hasn't taken yet and keeps what its own passes wrote. The whole root goes
up, over what the pass wrote, when raw bytes replace it, and when the render
creates the buffer or recreates it at a new size, as two shaders that declare
the root at different sizes make it do. Two shaders that declare it at the
same size share one buffer. The first render of either on a buffer the other
created uploads each field and item it ever set.

`read_storage` is `async` because the underlying `map_async` requires an
awaited callback. It returns a Promise in JS, is `async` in Swift and
`suspend` in Kotlin, and blocks until the bytes arrive in Python. On native
the implementation drives the device forward (`device.poll(Wait)`) before
awaiting, so callers don't have to spin their own poll loop. On Web the
browser schedules the callback; the helper just awaits.

The result holds exactly as many bytes as the binding spans, also for a
root of `f16` values, which can span 2 or 6 bytes and reads back as 2-byte
IEEE halves.

Returns `RendererError::StorageBindingNotFound` when the shader does not
declare a storage binding called `binding`, or when no render pass has yet
materialized the GPU-side buffer (storage buffers are allocated lazily on
first bind).

In JavaScript, `readStorage` waits for the frames rendered before it to
report their GPU errors and throws the first one, as
[`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render)
describes. An error throws only once, so when another call, such as the next
`render`, has already thrown it, `readStorage` returns the bytes the failed
frame left behind.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# use bytemuck;
use fragmentcolor::{Pass, Renderer, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;

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
pass.add_shader(&compute)?;
renderer.render(&pass, &target)?;

let bytes = renderer.read_storage(&compute, "out").await?;
# let values: &[f32] = bytemuck::cast_slice(&bytes);
# assert_eq!(values, &[1.0, 2.0, 3.0, 4.0]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
