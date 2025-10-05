# Target::resize(size: Size { width, heigth } | (u32, u32) | [u32; 2])

Resizes the [Target](https://fragmentcolor.org/api/core/target) to the given width and height.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();
let mut target = renderer.create_texture_target([64, 32]).await?;

target.resize([128, 64]);

# let size: [u32; 2] = target.size().into();
# assert_eq!(size, [128, 64]);
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
