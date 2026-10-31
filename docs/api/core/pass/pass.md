# Pass

The [Pass](https://fragmentcolor.org/api/core/pass) object is a collection of [Shader](https://fragmentcolor.org/api/core/shader) objects that are rendered to a [Target](https://fragmentcolor.org/api/targets/target) by the [Renderer](https://fragmentcolor.org/api/core/renderer).

While the [Shader](https://fragmentcolor.org/api/core/shader) represents a single **Render Pipeline** or a **Compute Pipeline**,
the [Pass](https://fragmentcolor.org/api/core/pass) can be used to draw multiple Shaders in sequence,
for example when you have multiple objects in a scene with different materials.

The [Pass](https://fragmentcolor.org/api/core/pass) represents a single RenderPass or a ComputePass in the WebGPU API.

The constructor creates a RenderPass by default. To create a ComputePass, call [Pass](https://fragmentcolor.org/api/core/pass)::compute().

The first [Shader](https://fragmentcolor.org/api/core/shader) added sets the pass's kind, and the pass then takes only shaders of that kind.
[`add_shader`](https://fragmentcolor.org/api/core/pass#add_shader) returns `PassError::ShaderKind` for a compute shader on a render pass, or the reverse, and leaves the pass as it was.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let canvas = fragmentcolor::HeadlessWindow::new([100, 100]);
# /*[js]
# const canvas = document.createElement("canvas");
# */
# /*[python]
# import os, sys
# # Skip this example in headless/CI environments that have no display surface.
# if os.environ.get('DISPLAY') is None and sys.platform != 'win32' and os.environ.get('FC_ALLOW_WINDOW') != '1':
#     raise SystemExit(0)
# from rendercanvas.auto import RenderCanvas
# canvas = RenderCanvas(size=(100, 100))
# */
# /*[swift]
# import QuartzCore
# let canvas = CAMetalLayer()
# */
# /*[kotlin]
# val surfaceTexture = android.graphics.SurfaceTexture(0)
# val canvas = android.view.Surface(surfaceTexture)
# */

use fragmentcolor::{ Shader, Pass, Renderer };

let renderer = Renderer::new();
let target = renderer.create_target(canvas).await?;
let shader = Shader::default();

let mut pass = Pass::new("First Pass");
pass.add_shader(&shader)?;

let mut pass2 = Pass::new("Second Pass");
pass2.add_shader(&shader)?;

// standalone
renderer.render(&pass, &target)?;

// vector of passes rendered in order (any iterable of Pass is renderable)
renderer.render(&vec![pass, pass2], &target)?;
# /*[kotlin]
# surfaceTexture.release()
# */

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
