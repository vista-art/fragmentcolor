# TextureMipChain::format

Return the wgpu texture format the chain was prepared for. Matches the `format` argument passed to `prepare(...)`.

## Example

```rust
use fragmentcolor::{TextureFormat, TextureMipChain};

let pixels: Vec<u8> = vec![200; 4 * 4 * 4];
let chain = TextureMipChain::prepare((
    pixels.as_slice(),
    TextureFormat::Rgba8UnormSrgb,
    [4u32, 4u32],
))?;
let _ = chain.format();
# Ok::<(), Box<dyn std::error::Error>>(())
```
