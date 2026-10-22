# WindowTarget::get_image()

This operation returns an empty array for [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget).

Use a [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) instead.

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
use fragmentcolor::{Renderer, Shader, Target};

let renderer = Renderer::new();
let target = renderer.create_target(canvas).await?;
renderer.render(&Shader::default(), &target)?;

let image = target.get_image().await;
# _ = image;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
