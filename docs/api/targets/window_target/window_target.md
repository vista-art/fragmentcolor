# WindowTarget

[WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) is an implementation of [Target](https://fragmentcolor.org/api/core/target) that represents a rendering destination attached to a platform-specific window.

The [Target](https://fragmentcolor.org/api/core/target) object is a rendering destination for the [Renderer](https://fragmentcolor.org/api/core/renderer).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget)).

## Example

```no-run
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
    
use fragmentcolor::{Renderer, Shader};

// Platform-specific window, e.g. winit, glfw, sdl2, etc.
// We have official support for winit but other libraries can be used
// if you implement the required traits. See the source code for details.
let window = fragmentcolor::mock_window([800, 600]);

let renderer = Renderer::new();
let target = renderer.create_target(window).await?;

renderer.render(&Shader::default(), &target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
