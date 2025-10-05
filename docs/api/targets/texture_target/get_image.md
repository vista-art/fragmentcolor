# TextureTarget::get_image()

Returns the current contents of the target as a byte array in RGBA8 format.

## Example

```rust
use fragmentcolor::{Renderer, Target, Shader};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
renderer.render(&Shader::default(), &target)?;

let image = target.get_image();

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
