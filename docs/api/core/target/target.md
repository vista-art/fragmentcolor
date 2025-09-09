# Target

The [Target](https://fragmentcolor.org/api/core/target) interface (trait in Rust) is a rendering destination for the [Renderer](https://fragmentcolor.org/api/core/renderer) implemented by both [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) and [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget)).

[Target](https://fragmentcolor.org/api/core/target) constructors are private and can only be created via the [Renderer](https://fragmentcolor.org/api/core/renderer) using either `Renderer.create_target(window)` for on-screen rendering, or `Renderer.create_texture_target()` for offscreen rendering.

## Examples

### WindowTarget (on-screen)

```rust
use fragmentcolor::{Renderer, Shader};
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let window = fragmentcolor::headless_window([800, 600]);
let target = renderer.create_target(window).await?;
let shader = Shader::default();
renderer.render(&shader, &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

### TextureTarget (offscreen)

```rust
use fragmentcolor::{Renderer, Shader, Target};
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;
let shader = Shader::default();
renderer.render(&shader, &target)?;
let image = target.get_image();

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
