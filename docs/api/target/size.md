# size() -> [u32; 2]

Returns the size of the [Target](https://fragmentcolor.org/api/target) in pixels.

## Example

```rust
use fragmentcolor::{Renderer, Target};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 32]).await?;
let size: [u32; 2] = target.size().into();
assert_eq!(size, [64, 32]);

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
