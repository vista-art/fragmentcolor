# Target::size() -> Size { width, height }

Returns the size of the [Target](https://fragmentcolor.org/api/core/target) in pixels.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 32]).await?;
let size = target.size();
let width = size.width;
let height = size.height;
let depth = size.depth;

# assert_eq!(size.width, 64);
# assert_eq!(size.height, 32);
# assert_eq!(size.depth, Some(1));
# let size: [u32; 2] = target.size().into();
# assert_eq!(size, [64, 32]);
# let size: [u32; 3] = target.size().into();
# assert_eq!(size, [64, 32, 1]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
