# TextureTarget::size() -> Size { width: u32, height: u32, depth: u32 }

Returns the size of the [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) in pixels.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Renderer;

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;
let size = target.size();
let width = size.width;
let height = size.height;
let depth = size.depth;

# assert_eq!(target.size(), [64, 64]);
# assert_eq!(width, 64);
# assert_eq!(height, 64);
# assert_eq!(depth, 1);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
