# AndroidTextureTarget

Android wrapper around the headless [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget). Implements the [Target](https://fragmentcolor.org/api/core/target) interface and is returned by [Renderer](https://fragmentcolor.org/api/core/renderer)::create_texture_target_android.

Use this when you need an offscreen texture render target on Android (outside of a window/surface-backed target).

## Example

```rust
# fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Renderer;

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64])?;

let size = target.size();

# assert_eq!(size.width, 64);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
