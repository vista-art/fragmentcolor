# Pass::set_target(target)

Attach a per-pass color render target. When set, this pass renders into the provided texture instead of the final Target.

A Pass has at most one color target — call `set_target` again to swap it. Use this to render intermediate results (e.g., a shadow map) that later passes can sample.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, Pass, TextureFormat};

let r = Renderer::new();
let tex_target = r.create_texture_target([512, 512]).await?;

let p = Pass::new("shadow");
p.set_target(&tex_target)?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
