# get_current_frame() -> TargetFrame

Returns a frame wrapper for the offscreen texture.

Most users do not need to call this directly; the [Renderer](https://fragmentcolor.org/api/renderer) uses it internally.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Renderer;

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
let frame = target.get_current_frame()?;
let _format = frame.format();

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
