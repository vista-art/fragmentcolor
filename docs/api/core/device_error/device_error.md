# DeviceError

A GPU error that an error scope caught. [`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render) returns one when a pipeline fails to build or a frame's commands fail, in place of drawing nothing without a word.

It holds three values, under the same names in every language:

- `kind`, an [ErrorKind](https://fragmentcolor.org/api/core/errorkind): `Validation`, `Internal` or `Memory`.
- `label`, the step that raised it: `render pipeline`, `compute pipeline`, `light binning pipeline` or `frame`, and in Rust, Python, Swift and Kotlin also a bind group or texture view inside the frame.
- `detail`, the device's own words about what went wrong.

Each language raises it in its own form:

- Rust returns `RendererError::DeviceError { kind, label, detail }`. On the web a readback, such as `get_image`, returns `TextureError::DeviceError` with the same fields.
- JavaScript throws an `Error` with `kind`, `label` and `detail` properties.
- Python raises `DeviceError`, a `FragmentColorError`, with `kind`, `label` and `detail` attributes.
- Swift throws `FragmentColorError.Device(kind:label:detail:)`.
- Kotlin throws `FragmentColorException.Device(kind, label, detail)`.

Each language's own text for the error carries all three, as `Device error (validation) in 'frame': ...`: `Error.message` in JavaScript, `str(e)` in Python, `localizedDescription` and `description` in Swift, and `toString()` in Kotlin. Kotlin's `Throwable.message` lists the fields instead, as `kind=VALIDATION, label=frame, detail=...`.

On the web the GPU reports an error after `render` has returned, so the next call throws it, as `Renderer::render` describes.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
let shader = Shader::default();

// A GPU error in this frame returns as a DeviceError.
renderer.render(&shader, &target)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
