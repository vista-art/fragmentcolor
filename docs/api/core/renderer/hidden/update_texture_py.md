# Renderer.update_texture(id, bytes)

Python wrapper for `Renderer::update_texture`.

## Example

```python
from fragmentcolor import Renderer, TextureFormat

renderer = Renderer()
texture = renderer.create_storage_texture([64, 64], TextureFormat.Rgba, None)
id = texture.id()
frame = bytes(64 * 64 * 4)

renderer.update_texture(id, frame)
```
