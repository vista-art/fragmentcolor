# Renderer::new()

## Creates a new Renderer

The [Renderer](https://fragmentcolor.org/api/core/renderer) internals are lazily initialized
when the user creates a [Target](https://fragmentcolor.org/api/core/target) or renders a Bitmap.
This ensures the adapter and device are compatible with the target environment.

At the point of creation, we don't know if it will be used offscreen or attached to a Window.

The API ensures the [Renderer](https://fragmentcolor.org/api/core/renderer) is usable when `render()` is called,
because the `render()` method expects a [Target](https://fragmentcolor.org/api/core/target) as input, and
the only way to create a [Target](https://fragmentcolor.org/api/core/target)
is by calling `renderer.create_target(Window)` first.

- `Renderer.create_target()` internally initializes a window adapter, while
- `Renderer.render_image()` initializes an offscreen adapter.

## Example

```rust
use fragmentcolor::Renderer;
let renderer = Renderer::new();
```
