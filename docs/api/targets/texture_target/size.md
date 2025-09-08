# size() -> [u32; 2]

Returns the size of the [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) in pixels.

## Example

```rust
use fragmentcolor::Renderer;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([64, 64]).await?;
assert_eq!(target.size(), [64, 64]);

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
