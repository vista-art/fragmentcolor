# Target::get_image()

Returns the current contents of the target as a byte array in RGBA8 format.

A [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) returns its pixels, or an error when the readback fails. A [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) always returns an error, because a frame on screen can't be read back.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Shader, Renderer, Target};

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
renderer.render(&Shader::default(), &target)?;

let image = target.get_image().await?;

# assert_eq!(image.len(), 16 * 16 * 4); // RGBA8
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
