# WindowTarget

[WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) is an implementation of [Target](https://fragmentcolor.org/api/core/target) that represents a rendering destination attached to a platform-specific window.

The [Target](https://fragmentcolor.org/api/core/target) object is a rendering destination for the [Renderer](https://fragmentcolor.org/api/core/renderer).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget)).

## Example

```rust,no_run
use fragmentcolor::{Renderer, Shader};
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let window = fragmentcolor::mock_window([800, 600]);
let target = renderer.create_target(window).await?;
let shader = Shader::default();
renderer.render(&shader, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
