# WindowTarget::resize(size: [u32; 2] | (u32, u32))

Resizes the [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) to the given width and height.
Call it when the window or canvas changes size.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let canvas = fragmentcolor::headless_window([800, 600]);
# /*[js]
# const canvas = document.createElement("canvas");
# */
# /*[python]
# import os, sys
# # Skip this example in headless/CI environments that have no display surface.
# if os.environ.get('DISPLAY') is None and sys.platform != 'win32' and os.environ.get('FC_ALLOW_WINDOW') != '1':
#     raise SystemExit(0)
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
let mut target = renderer.create_target(canvas).await?;

target.resize([1024, 768]);

# let size = target.size();
# assert_eq!([size.width, size.height], [1024, 768]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
