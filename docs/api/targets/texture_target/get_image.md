# TextureTarget::get_image()

Returns the current contents of the target as a byte array in RGBA8 format.

Rows run from top to bottom with no padding, so the pixel at `(x, y)` starts at byte `(y * width + x) * 4`.

Python returns a NumPy array of shape `(height, width, 4)`, indexed `image[y, x]`. That is the layout `Renderer.create_texture` reads, so `renderer.create_texture(target.get_image())` uploads the same image.

A readback that fails or times out returns an error, a thrown `Error` in JavaScript and a `FragmentColorError` in Python, Swift and Kotlin.

## Example

```rust
use fragmentcolor::{Renderer, Target, Shader};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {

let renderer = Renderer::new();
let target = renderer.create_texture_target([16, 16]).await?;
renderer.render(&Shader::default(), &target)?;

let image = target.get_image().await?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
