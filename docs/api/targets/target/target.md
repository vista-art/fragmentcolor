# Target

The [Target](https://fragmentcolor.org/api/core/target) interface (trait in Rust) is a rendering destination for the [Renderer](https://fragmentcolor.org/api/core/renderer) implemented by both [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) and [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget)).

[Target](https://fragmentcolor.org/api/core/target) constructors are private and can only be created via the [Renderer](https://fragmentcolor.org/api/core/renderer) using either `Renderer.create_target(window)` for on-screen rendering, or `Renderer.create_texture_target()` for offscreen rendering.

## Examples

### WindowTarget (on-screen)

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader};

let renderer = Renderer::new();

// Use your platform's windowing system to create a window.
// We officially support Winit. Check the examples folder for details.
let window = fragmentcolor::headless_window([800, 600]);

let target = renderer.create_target(window).await?;

// To animate, render again in your event loop...
renderer.render(&Shader::default(), &target)?;
renderer.render(&Shader::default(), &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

### TextureTarget (offscreen)

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader, Target};

let renderer = Renderer::new();

// Creates a target Texture
let target = renderer.create_texture_target([64, 64]).await?;

let shader = Shader::default();
renderer.render(&shader, &target)?;

// Read back the rendered image (byte array of RGBA8 pixels)
let image = target.get_image();

# assert_eq!(image.len(), 64 * 64 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
