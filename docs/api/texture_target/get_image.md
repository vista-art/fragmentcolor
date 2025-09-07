# get_image()

Returns the current contents of the target as a byte array in RGBA8 format.

## Example

```rust
use fragmentcolor::Renderer;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let renderer = Renderer::new();
let target = pollster::block_on(renderer.create_texture_target([16, 16]))?;
renderer.render(&fragmentcolor::Shader::default(), &target)?;

let image = target.get_image();
# Ok(())
# }
```
