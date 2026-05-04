# Renderer::read_texture (Python)

Python wrapper for `Renderer::read_texture`. Blocks the Python thread synchronously (via `pollster::block_on`) and returns the tightly-packed pixel bytes for the registered texture in its native format.

## Example

```python
from fragmentcolor import Renderer, TextureFormat
renderer = Renderer()
texture = renderer.create_storage_texture([64, 64], TextureFormat.Rgba)
texture.write([0] * (64 * 64 * 4))

bytes = renderer.read_texture(texture.id())
```
