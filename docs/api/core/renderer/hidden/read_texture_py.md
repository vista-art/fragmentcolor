# Renderer::read_texture (Python)

In Python, read back texture pixels via the texture object's get_image() method.

## Example

```python
from fragmentcolor import Renderer, TextureFormat
renderer = Renderer()
texture = renderer.create_storage_texture([64, 64], TextureFormat.Rgba)
texture.write([0] * (64 * 64 * 4))

bytes = texture.get_image()
```
