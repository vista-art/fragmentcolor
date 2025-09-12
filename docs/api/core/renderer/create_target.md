# Renderer::create_target(target: Canvas | Window)

Creates a [Target](https://fragmentcolor.org/api/core/target) attached to a platform-specific canvas or window.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();

// Use your platform's windowing system to create a window.
// We officially support Winit. Check the examples folder for details.
let window = fragmentcolor::headless_window([800, 600]);

let target = renderer.create_target(window).await?;

# let s = target.size();
# assert_eq!([s.width, s.height], [800, 600]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
