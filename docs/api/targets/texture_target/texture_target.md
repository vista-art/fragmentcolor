# TextureTarget

The [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) is an offscreen [Target](https://fragmentcolor.org/api/core/target) backed by a GPU texture.

Use it for headless rendering, tests, server-side image generation, or CI.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Shader, Target};

let renderer = Renderer::new();
let mut target = renderer.create_texture_target([64, 64]).await?;

let shader = Shader::default();
renderer.render(&shader, &target)?;

let image = target.get_image();

# assert_eq!(image.len(), 64 * 64 * 4); // RGBA8
# target.resize([128, 128]);
# assert_eq!(target.size().width, 128);
# assert_eq!(target.size().height, 128);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
