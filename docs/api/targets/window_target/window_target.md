# WindowTarget

[WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) is an implementation of [Target](https://fragmentcolor.org/api/core/target) that represents a rendering destination attached to a platform-specific window.

The [Target](https://fragmentcolor.org/api/core/target) object is a rendering destination for the [Renderer](https://fragmentcolor.org/api/core/renderer).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget)).

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader, Target};

// Use your platform's windowing system to create a window.
// We officially support Winit. Check the examples folder for details.
let window = fragmentcolor::headless_window([800, 600]);

let renderer = Renderer::new();
let target = renderer.create_target(window).await?;

renderer.render(&Shader::default(), &target)?;

# let s = target.size();
# assert_eq!([s.width, s.height], [800, 600]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
