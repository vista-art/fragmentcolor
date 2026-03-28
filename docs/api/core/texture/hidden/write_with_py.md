# Texture.write_with(bytes, options)

Python wrapper for `Texture::write_with`.

## Example

```python
from fragmentcolor import Renderer, TextureFormat, TextureWriteOptions

renderer = Renderer()
texture = renderer.create_storage_texture([64, 32], TextureFormat.Rgba, None)
frame = bytes(64 * 32 * 4)
opt = TextureWriteOptions.whole().with_bytes_per_row(256).with_rows_per_image(32)

texture.write_with(frame, opt)
```
