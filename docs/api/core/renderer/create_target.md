# Renderer::create_target(target: Canvas | Window)

Creates a [Target](https://fragmentcolor.org/api/targets/target) attached to a platform-specific canvas or window.

In Rust, we officially support Winit. Check the examples folder for details.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let canvas = fragmentcolor::headless_window([800, 600]);
# /*[js]
# const canvas = document.createElement("canvas");
# */
# /*[python]
# from rendercanvas.auto import RenderCanvas
# canvas = RenderCanvas(size=(800, 600))
# */
# /*[swift]
# import QuartzCore
# let canvas = CAMetalLayer()
# */
# /*[kotlin]
# val canvas = android.view.Surface(android.graphics.SurfaceTexture(0))
# */

use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();

// The canvas comes from your platform's windowing system.
let target = renderer.create_target(canvas).await?;

# let s = target.size();
# assert_eq!([s.width, s.height], [800, 600]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
