# WindowTarget::resize(size: [u32; 2] | (u32, u32))

Resizes the [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) to the given width and height.
Call it when the window or canvas changes size.

A width or height of zero, such as the 0×0 a minimized window reports, is no error: the target keeps
its last size and draws at the new one once the window has an area again. A side past the device limit
returns an error.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let canvas = fragmentcolor::HeadlessWindow::new([800, 600]);
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
# val surfaceTexture = android.graphics.SurfaceTexture(0)
# val canvas = android.view.Surface(surfaceTexture)
# */
use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();
let mut target = renderer.create_target(canvas).await?;

target.resize([1024, 768])?;
# /*[kotlin]
# surfaceTexture.release()
# */

# let size = target.size();
# assert_eq!([size.width, size.height], [1024, 768]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
