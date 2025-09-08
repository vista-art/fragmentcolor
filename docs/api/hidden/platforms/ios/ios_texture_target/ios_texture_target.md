# IosTextureTarget

iOS wrapper around the headless [TextureTarget](https://fragmentcolor.org/api/texture_target). Implements the [Target](https://fragmentcolor.org/api/target) interface and is returned by [Renderer](https://fragmentcolor.org/api/renderer)::create_texture_target_ios.

Use this when you need an offscreen texture render target on iOS (outside of a view/surface-backed target).

## Example

```rust
use fragmentcolor::Renderer;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target_ios([64, 64]))?;

let size = target.size();
let _ = size.width; // placeholder usage
# Ok(())
# }
```

