# resize(size: [u32; 2] | (u32, u32))

Resizes the [Target](https://fragmentcolor.org/docs/api/target) to the given width and height.

## Example

```rust
use fragmentcolor::Renderer;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let mut target = pollster::block_on(renderer.create_texture_target([64, 32]))?;

assert_eq!(target.size(), [64, 32]);

target.resize([128, 64]);
assert_eq!(target.size(), [128, 64]);
# Ok(())
# }
```
