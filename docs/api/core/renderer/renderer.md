# Renderer

The [Renderer](https://fragmentcolor.org/api/core/renderer) is the main entry point for
[FragmentColor](https://fragmentcolor.org) and normally the first object you create.

It is used to render
[Shaders](https://fragmentcolor.org/api/core/shader) and
[Passes](https://fragmentcolor.org/api/core/pass) (single passes or any iterable of them)
to a [Target](https://fragmentcolor.org/api/targets/target) (canvas, window, or texture).

The [Renderer](https://fragmentcolor.org/api/core/renderer) internals are lazily initialized
when the user creates a [Target](https://fragmentcolor.org/api/targets/target).

See the constructor [Renderer::new()](https://fragmentcolor.org/api/core/renderer#new)
description below for details.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let canvas = fragmentcolor::HeadlessWindow::new([800, 600]);
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
# val surfaceTexture = android.graphics.SurfaceTexture(0)
# val canvas = android.view.Surface(surfaceTexture)
# */

use fragmentcolor::{Shader, Renderer, Target};

let renderer = Renderer::new();

// The canvas comes from your platform's windowing system
let target = renderer.create_target(canvas).await?;
let texture_target = renderer.create_texture_target([16, 16]).await?;

// RENDERING
renderer.render(&Shader::default(), &texture_target)?;

// That's it. Welcome to FragmentColor!
# /*[kotlin]
# surfaceTexture.release()
# */

# let s = target.size();
# assert_eq!([s.width, s.height], [800, 600]);
# let s2 = texture_target.size();
# assert_eq!([s2.width, s2.height], [16, 16]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```

## Methods
