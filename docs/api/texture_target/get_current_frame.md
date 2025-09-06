# get_current_frame() -> TargetFrame

Returns a frame wrapper for the offscreen texture.

Most users do not need to call this directly; the [Renderer](https://fragmentcolor.org/docs/api/renderer) uses it internally.

## Example

```rust
use fragmentcolor::Renderer;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([16, 16]))?;
let frame = target.get_current_frame()?;
let _format = frame.format();
# Ok(())
# }
```
