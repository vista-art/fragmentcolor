# Renderer::wait()

Block the current thread until every submission queued on this renderer's device has finished executing.

Use this as an explicit synchronization point between `render()` bursts and readbacks — for example before `Renderer::read_texture`, `Texture::get_image`, or `TextureTarget::get_image` — when you need deterministic ordering rather than the "submission order" the driver normally guarantees.

- Native only (on WASM it is a no-op — the browser drives readiness via its own mapping lifecycle).
- Returns after the device reports idle or a 5-second timeout.

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
