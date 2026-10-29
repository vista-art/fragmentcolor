# WindowTarget::get_image()

Returns an error for a [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget): a frame on screen can't be read back. JavaScript throws an `Error` named `FragmentColorError`, Python and Swift throw a `FragmentColorError`, and Kotlin throws a `FragmentColorException`.

To read the pixels of a frame, draw it into a [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) and call `get_image` there, as the example does.

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
# val surfaceTexture = android.graphics.SurfaceTexture(0)
# val canvas = android.view.Surface(surfaceTexture)
# */
use fragmentcolor::{Renderer, Shader, Target};

let renderer = Renderer::new();
let target = renderer.create_target(canvas).await?;
let shader = Shader::default();
renderer.render(&shader, &target)?;

// The same frame, drawn where it can be read back.
let offscreen = renderer.create_texture_target([800, 600]).await?;
renderer.render(&shader, &offscreen)?;
let image = offscreen.get_image().await?;
# /*[kotlin]
# surfaceTexture.release()
# */
# assert_eq!(image.len(), 800 * 600 * 4);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
