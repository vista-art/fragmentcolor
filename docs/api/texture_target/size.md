# size() -> [u32; 2]

Returns the size of the [TextureTarget](https://fragmentcolor.org/api/texture_target) in pixels.

## Example

```rust
use fragmentcolor::Renderer;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([64, 64]))?;
assert_eq!(target.size(), [64, 64]);
# Ok(())
# }
```
