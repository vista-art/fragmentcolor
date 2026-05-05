# Renderer::new()

Create a new [Renderer](https://fragmentcolor.org/api/core/renderer).

The renderer's GPU adapter and device are initialized lazily on the first
target you create, so the same `Renderer` works whether you end up
rendering offscreen or attaching it to a window. By the time `render()`
is called the GPU resources are already in place — `render` requires a
[Target](https://fragmentcolor.org/api/core/target), and the only way to
build one is through this renderer:

- `renderer.create_target(Window)` for an on-screen target, or
- `renderer.create_texture_target([w, h])` for offscreen rendering.

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
