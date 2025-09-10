# Target::get_current_frame() -> TargetFrame

Returns a frame wrapper containing the texture view to render and the target format.

Most users do not need to call this directly; the [Renderer](https://fragmentcolor.org/api/core/renderer) uses it internally.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Renderer, Target};

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
let frame = target.get_current_frame()?; // Acquire a frame
let _format = frame.format();

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
