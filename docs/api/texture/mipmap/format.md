# Mipmap::format

Return the wgpu texture format the chain was prepared for. Matches the `format` argument passed to `build(...)`.

## Example

```rust
use fragmentcolor::{Mipmap, TextureFormat};

let pixels: Vec<u8> = vec![200; 4 * 4 * 4];
let chain = Mipmap::build((
    pixels.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [4, 4],
))?;
let _ = chain.format();
# Ok::<(), Box<dyn std::error::Error>>(())
```
