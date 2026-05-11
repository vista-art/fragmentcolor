# Mipmap::format

Return the wgpu texture format the chain was built for — the same `format` you passed to `build(...)`. Useful when you receive a chain you didn't build yourself (passed in from another thread or stored in a cache) and need to confirm it matches the format your shader expects before uploading.

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
