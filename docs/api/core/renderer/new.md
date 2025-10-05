# Renderer::new()

## Creates a new Renderer

At the point of creation, we don't know the [Renderer](https://fragmentcolor.org/api/core/renderer) will be used offscreen or attached to a Window.

So, the rendering internals are lazily initialized
when the user creates a [Target](https://fragmentcolor.org/api/core/target).
This ensures the adapter and device are compatible with the environment.

The API ensures the [Renderer](https://fragmentcolor.org/api/core/renderer) is usable when `render()` is called,
because the `render()` method expects a [Target](https://fragmentcolor.org/api/core/target) as input, and
the only way to create a [Target](https://fragmentcolor.org/api/core/target)
is by first calling:

- `renderer.create_target(Window)` to create a window adapter, or
- `renderer.create_texture_target()` to create a target texture

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();
let texture_target = renderer.create_texture_target([16, 16]).await?;

# let s = texture_target.size();
# assert_eq!([s.width, s.height], [16, 16]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
