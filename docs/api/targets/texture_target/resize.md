# TextureTarget::resize(size: [u32; 2] | (u32, u32))

Resizes the [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) to the given width and height.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Renderer;

let renderer = Renderer::new();
let mut target = renderer.create_texture_target([64, 64]).await?;

target.resize([128, 32]);

# assert_eq!(target.size(), [128, 32]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
