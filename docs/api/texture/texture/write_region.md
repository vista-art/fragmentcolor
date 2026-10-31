# Texture::write_region(bytes, region)

Same as `Texture::write`, but writes into a sub-region of the texture and/or with explicit source data layout.

The `region` argument is required; use `Texture::write` to replace the whole texture. An array, tuple or `TextureRegion` with a width, height or depth of 0 is an `InvalidInput` error. It accepts anything convertible into a `TextureRegion`:
- `[w, h]` / `(w, h)`: a `w` by `h` rectangle at origin `(0, 0, 0)`
- `[x, y, w, h]` / `(x, y, w, h)`: 2D rectangle
- `[x, y, z, w, h, d]`: 3D box (for layered or 3D textures)
- A `ScreenRegion`: its pixel rectangle at `z = 0`, one layer deep. A region built field by field can be invalid: one whose max lies before its min, or with a width or height of 0, returns `TextureError::Region` with `ScreenRegionError::Reversed` or `ScreenRegionError::Empty`
- A `TextureRegion` constructed explicitly with `.with_stride(...)` / `.with_rows(...)` for advanced data-layout control

## Notes
- See `Texture::write` for format and row layout details.
- `bytes_per_row` (set via `.with_stride(...)`) defaults to the region width times the bytes per pixel, or to that row padded to 256 bytes when the data is exactly that size. Without it, data of any other length is an `InvalidInput` error. A larger value skips the padding at the end of each source row, and the data may then run past the region.
- `rows_per_image` (set via `.with_rows(...)`) defaults to the region height.
- A texture with a mip chain, which `create_texture` builds by default, takes whole writes only, since a partial one would leave the smaller levels stale. Any other region is an `InvalidInput` error, so create a texture you update in parts with `mipmaps: false`, or from a KTX2 file with one level.

## Example
```rust
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
use fragmentcolor::{Renderer, TextureFormat, TextureRegion};
let renderer = Renderer::new();
let texture = renderer.create_storage_texture(([64, 32], TextureFormat::Rgba)).await?;
let bytes = vec![0; 64 * 32 * 4];

// Simple sub-rectangle update.
texture.write_region(&bytes, [0, 0, 64, 32])?;

// Explicit data layout, for when the source rows are padded.
let region = TextureRegion::from([0, 0, 64, 32])
    .with_stride(256)
    .with_rows(32);
texture.write_region(&bytes, region)?;
# Ok(())
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> { pollster::block_on(run()) }
```
