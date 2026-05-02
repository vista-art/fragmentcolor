# Texture::write_region(bytes, region)

Same as `Texture::write`, but writes into a sub-region of the texture and/or with explicit source data layout.

The `region` argument accepts anything convertible into a `TextureRegion`:
- `[w, h]` / `(w, h)` — full size, origin `(0, 0, 0)`
- `[x, y, w, h]` / `(x, y, w, h)` — 2D rectangle
- `[x, y, z, w, h, d]` — 3D box (for layered or 3D textures)
- A `TextureRegion` constructed explicitly with `.with_stride(...)` / `.with_rows(...)` for advanced data-layout control

## Notes
- See `Texture::write` for format and alignment details.
- `bytes_per_row` (set via `.with_stride(...)`) must be a multiple of 256 when provided.
- `rows_per_image` (set via `.with_rows(...)`) defaults to the region height.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat, TextureRegion};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64u32, 32u32], TextureFormat::Rgba)).await?;
let bytes = vec![0u8; 64 * 32 * 4];

// Simple sub-rectangle update.
texture.write_region(&bytes, [0, 0, 64, 32])?;

// Explicit data layout (advanced — when source rows are padded).
let region = TextureRegion::from([0, 0, 64, 32])
    .with_stride(256)
    .with_rows(32);
texture.write_region(&bytes, region)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
