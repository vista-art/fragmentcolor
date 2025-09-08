# get_image()

This operation returns an empty array for [WindowTarget](https://fragmentcolor.org/api/window_target).

Use a [TextureTarget](https://fragmentcolor.org/api/texture_target) instead.

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
