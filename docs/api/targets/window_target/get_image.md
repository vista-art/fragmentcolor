# WindowTarget::get_image()

This operation returns an empty array for [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget).

Use a [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) instead.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Target, Shader};

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
renderer.render(&Shader::default(), &target)?;

let image = target.get_image();

# assert_eq!(image.len(), 16 * 16 * 4); // RGBA8
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
