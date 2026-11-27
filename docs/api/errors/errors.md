# Errors

A call that fails returns an error in Rust and throws one in JavaScript, Python, Swift and Kotlin. Every error has a class, which says what went wrong, a message, and the fields of its class, under the same names in every language.

Rust returns the error type of the call, such as `RendererError` from [`Renderer::render`](https://fragmentcolor.org/api/core/renderer#render), and each class below names the variants it covers. A variant with fields holds them by name, as `RendererError::DeviceError { kind, label, detail }` does. The other languages raise the class itself:

- JavaScript throws an `Error` whose `name` is the class, such as `DeviceError`, with each field as a property.
- Python raises the class, with each field as an attribute. Every class is a `FragmentColorError`, so `except FragmentColorError` catches them all. A few are also a builtin exception, as their entries below say.
- Swift throws `FragmentColorError`, an enum with a case for each class. A case takes the class name without its `Error` suffix, such as `.Device`, and `.Other` stands for the base class. Each case holds the `message` and the fields, and `name` returns the class name.
- Kotlin throws `FragmentColorException`, with a subclass for each class under Swift's case names, such as `FragmentColorException.Device`. Each one holds the `message` and the fields, and `name` returns the class name.

The message is the same text everywhere: `Error.message` in JavaScript, `str(error)` in Python, `message`, `localizedDescription` and `description` in Swift, and `message` and `toString()` in Kotlin. Each field holds the same value in every language. In JavaScript `ErrorKind` is a numeric enum, and a `kind` is the number to compare with `ErrorKind.Memory` and the other values.

An argument of the wrong type, such as a string where a number goes, is a `TypeError` in JavaScript and Python. Swift and Kotlin refuse it when the code compiles.

## Classes

### FragmentColorError

The base class, for an error with no class of its own: WGSL that fails to parse, a uniform the shader doesn't declare, a vertex the shader can't read, a target from another `Renderer`, or a GPU that fails to open. In Rust that is every `ShaderError` and `MeshError`, and each variant of `RendererError`, `PassError`, `TextureError` and `InitializationError` that no other class names. Swift throws it as `.Other`, and Kotlin as `FragmentColorException.Other`.

### InvalidInputError

A value the call can't take: a size with a side of zero or past the device's limits, a color it can't read, a viewport with no area, texture bytes that don't match their size, or a number that isn't finite. In Rust that is every `SizeError`, `ColorError`, `ScreenRegionError` and `ModelError`, `TextureError::InvalidInput` and `TextureError::Region`, the `Color`, `Viewport`, `ViewportLimit`, `InvalidColorTarget` and `DispatchLimit` variants of `PassError`, and `LightError::NotFinite` and `LightError::ZeroDirection`.

JavaScript, Python, Swift and Kotlin also raise it before the call runs, for NaN, an infinity or a number past the 32-bit float range where a float goes. JavaScript raises it for a count that isn't a whole number in its type's range, and Python for an int count below 0 or past that range.

### UnsupportedMipmapFormatError

A texture format that the CPU mipmap builder can't fill, for a texture created with mipmaps. In Rust it is `TextureError::UnsupportedMipmapFormat`.

### MalformedImageError

Image bytes that can't be decoded, such as a truncated PNG. In Rust it is `TextureError::MalformedImageError`.

### OccludedSurfaceError

A window that is hidden or minimized, which leaves `render` no frame to draw on. Skip that frame and draw the next one. In Rust it is `RendererError::SurfaceError(SurfaceError::Occluded)`.

### DeviceError

A GPU error that an error scope caught. `Renderer::render` returns one when a pipeline fails to build or a frame's commands fail, in place of drawing nothing without a word. In Rust it is `RendererError::DeviceError`, and on the web a readback, such as `get_image`, returns `TextureError::DeviceError` with the same fields.

It carries three fields:

- `kind`, an `ErrorKind`, the kind of error the scope caught:
  - `Validation`: a call broke a rule of the GPU API, such as a bind group that doesn't match its layout.
  - `Internal`: the GPU or its driver failed for a reason of its own, such as a shader the backend compiler refuses.
  - `Memory`: the device ran out of memory.
- `label`, the step that raised it: `render pipeline`, `compute pipeline`, `light binning pipeline` or `frame`, and in Rust, Python, Swift and Kotlin also a bind group or texture view inside the frame.
- `detail`, the device's own words about what went wrong.

Running out of memory or an internal failure leaves an object invalid, and each later use of that object fails validation. So when one error causes others, the error that returns is the first in the order memory, internal, validation. Rust writes the kind as `ErrorKind::Memory`, JavaScript and Python as `ErrorKind.Memory`, Swift as `ErrorKind.memory` and Kotlin as `ErrorKind.MEMORY`.

The message carries all three, as in `Device error (validation) in 'frame': ...`. Swift's case is `.Device(message:kind:label:detail:)`.

On the web the GPU reports an error after `render` has returned, so the next call throws it, as `Renderer::render` describes.

### LimitError

Something in a shader or mesh past one of the device's limits, such as a bind group index past `max_bind_groups` or a vertex stride past `max_vertex_buffer_array_stride`. `Renderer::render` checks for it before it takes a frame. In Rust it is `RendererError::LimitError`, with four fields: `item`, what passes the limit, `limit`, the limit's name, `value`, what the item needs, and `max`, the device's maximum. The [Shader](https://fragmentcolor.org/api/core/shader#device-limits) page lists the checks.

### FeatureError

Something in a shader that needs a feature the device lacks, such as texture atomics or `f16`. In Rust it is `RendererError::FeatureError`, with two fields: `item`, what needs the feature, and `feature`, the wgpu flag the device lacks, such as `SHADER_F16`.

### BindingError

A texture uniform that was never set, or whose texture its slot can't take by format, usage, dimension, sample count or mip levels. In Rust it is `RendererError::BindingError`, with three fields: `uniform`, the uniform's name, `expected`, what the slot takes, and `found`, what it got.

### EntryError

A render shader without the `@vertex` or `@fragment` entry point its pass runs. In Rust it is `RendererError::EntryError`, with one field, `stage`, the entry point it lacks.

### SceneLoadError

A scene file that fails to load, such as a glTF file the parser refuses or one whose data FragmentColor can't use. In Rust it is every `SceneLoadError`. In Python it is also a `RuntimeError`.

### LightError

A light setter that doesn't fit the light's kind, such as a direction on a point light, or a range below zero. In Rust it is `LightError::FieldNotApplicable` and `LightError::NegativeRange`. In Python it is also a `ValueError`.

### CameraError

Camera parameters that leave no usable view, such as a field of view of zero or a near plane equal to the far one. In Rust it is every `CameraError`. In Python it is also a `ValueError`.

### MaterialError

A value a material setter or map can't use, such as UV set 2 or zero toon bands. A map's texture input that makes no texture raises the class `create_texture` raises for the same input. In Rust it is every other `MaterialError`. In Python it is also a `ValueError`.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{ErrorKind, Renderer, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
let shader = Shader::default();

// A GPU error in this frame returns as a DeviceError.
renderer.render(&shader, &target)?;

// The kind of GPU error a DeviceError carries.
let kind = ErrorKind::Memory;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
