# resize(size: [u32; 2] | (u32, u32))

Resizes the [Target](https://fragmentcolor.org/api/core/target) to the given width and height.

## Example

```rust
use fragmentcolor::{Renderer, Target};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let mut target = renderer.create_texture_target([64, 32]).await?;

let size: [u32; 2] = target.size().into();
assert_eq!(size, [64, 32]);

target.resize([128, 64]);
let size: [u32; 2] = target.size().into();
assert_eq!(size, [128, 64]);

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
