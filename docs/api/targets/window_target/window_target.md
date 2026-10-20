# WindowTarget

[WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) is an implementation of [Target](https://fragmentcolor.org/api/core/target) that represents a rendering destination attached to a platform-specific window.

The [Target](https://fragmentcolor.org/api/core/target) object is a rendering destination for the [Renderer](https://fragmentcolor.org/api/core/renderer).

It contains a GPU surface texture attached to a platform-specific window or an offscreen texture for headless rendering (see [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget)).

In Rust, we officially support Winit. Check the examples folder for details.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let canvas = fragmentcolor::headless_window([800, 600]);
# /*[js]
# const canvas = document.createElement("canvas");
# */
# /*[swift]
# import QuartzCore
# let canvas = CAMetalLayer()
# */
# /*[kotlin]
# val canvas = android.view.Surface(android.graphics.SurfaceTexture(0))
# */

use fragmentcolor::{Renderer, Shader, Target};

let renderer = Renderer::new();

// The canvas comes from your platform's windowing system.
let target = renderer.create_target(canvas).await?;

renderer.render(&Shader::default(), &target)?;

# let s = target.size();
# assert_eq!([s.width, s.height], [800, 600]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
