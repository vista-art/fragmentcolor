# Renderer::wait()

Block the calling thread until every submission queued on this renderer's device has finished executing.

Call `wait` between `render()` bursts and a readback — `Renderer::read_texture`, `Texture::get_image`, or `TextureTarget::get_image` — when you need deterministic ordering. Without it you get the driver's normal submission ordering, which is usually fine but doesn't guarantee a render is complete when the readback begins.

- Native only. On the web this is a no-op; the browser drives readiness through its own mapping lifecycle.
- Returns when the device reports idle, or after a 5-second timeout.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Shader, Target};

let r = Renderer::new();
let target = r.create_texture_target([8, 8]).await?;
let shader = Shader::default();
r.render(&shader, &target)?;
r.wait()?;
let _bytes = target.get_image().await;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
