# AndroidTextureTarget

Android wrapper around the headless [TextureTarget](https://fragmentcolor.org/api/texture_target). Implements the [Target](https://fragmentcolor.org/api/target) interface and is returned by [Renderer](https://fragmentcolor.org/api/renderer)::create_texture_target_android.

Use this when you need an offscreen texture render target on Android (outside of a window/surface-backed target).

## Example

```rust
use fragmentcolor::Renderer;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target_android([64, 64]))?;

let size = target.size();
assert_eq!(size.width, 64);
# Ok(())
# }
```
