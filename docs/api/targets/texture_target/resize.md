# TextureTarget::resize(size: [u32; 2] | (u32, u32))

Resizes the [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) to the given width and height.

Passes that draw into the target, shaders that sample it and every `texture()` handle use the new texture.
Its pixels start blank, and the current size keeps them. A width or height of zero logs a warning and keeps the texture.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();
let mut target = renderer.create_texture_target([64, 64]).await?;

target.resize([128, 32]);

# let size: [u32; 2] = target.size().into();
# assert_eq!(size, [128, 32]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
