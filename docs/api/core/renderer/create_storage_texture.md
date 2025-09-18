# Renderer::create_storage_texture

Create a storage-class texture for compute or image store/load.

- Default usage: STORAGE_BINDING | TEXTURE_BINDING | COPY_{SRC,DST}

## Example

```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {

use fragmentcolor::Renderer;
let r = Renderer::new();
let _tex = r
    .create_storage_texture([64, 64], wgpu::TextureFormat::Rgba8Unorm, None)
    .await?;

# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
