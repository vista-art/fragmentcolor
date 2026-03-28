# Renderer.unregister_texture(id)

Python wrapper for `Renderer::unregister_texture`.

## Example

```python
from fragmentcolor import Renderer, TextureFormat

renderer = Renderer()
texture = renderer.create_storage_texture([16, 16], TextureFormat.Rgba, None)
id = texture.id()

renderer.unregister_texture(id)
```
