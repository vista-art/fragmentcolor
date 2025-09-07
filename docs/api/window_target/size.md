# size() -> [u32; 2]

Returns the size of the [Target](https://fragmentcolor.org/api/target) in pixels.

## Example

```rust
use fragmentcolor::{Renderer, Shader};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([64, 32]))?;
assert_eq!(target.size(), [64, 32]);
# Ok(())
# }
```
