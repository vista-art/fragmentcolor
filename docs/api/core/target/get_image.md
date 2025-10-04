# Target::get_image() -> Vec<u8>

Returns the current contents of the target as a byte array in RGBA8 format.

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::{Shader, Renderer, Target};

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
renderer.render(&Shader::default(), &target)?;

let image = target.get_image();

# assert_eq!(image.len(), 16 * 16 * 4); // RGBA8
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
