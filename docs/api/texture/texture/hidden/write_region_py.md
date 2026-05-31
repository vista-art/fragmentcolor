# Texture.write_region(bytes, region)

Python wrapper for `Texture::write_region`.

The `region` argument accepts:
- `(w, h)` / `[w, h]`: size only, origin `(0, 0, 0)`
- `(x, y, w, h)` / `[x, y, w, h]`: 2D rectangle
- `(x, y, z, w, h, d)` / `[x, y, z, w, h, d]`: 3D box
- `{"x": ..., "y": ..., "z"?: ..., "width": ..., "height": ..., "depth"?: ..., "bytes_per_row"?: ..., "rows_per_image"?: ...}`
- `{"min_x": ..., "min_y": ..., "max_x": ..., "max_y": ..., "min_z"?: ..., "max_z"?: ..., "bytes_per_row"?: ..., "rows_per_image"?: ...}`

## Example

```python
from fragmentcolor import Renderer, TextureFormat

renderer = Renderer()
texture = renderer.create_storage_texture([64, 32], TextureFormat.Rgba)
bytes_data = bytes(64 * 32 * 4)

# Simple sub-rectangle update.
texture.write_region(bytes_data, [0, 0, 64, 32])

# Explicit data layout (advanced).
texture.write_region(bytes_data, {"x": 0, "y": 0, "width": 64, "height": 32, "bytes_per_row": 256, "rows_per_image": 32})
```
