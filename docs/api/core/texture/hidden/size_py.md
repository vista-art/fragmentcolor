# Texture::size (Python)

Return the Size object for this texture.

## Example

```python
from fragmentcolor import Renderer
renderer = Renderer()
pixels = [255, 255, 255, 255]
tex = renderer.create_texture(pixels, size=[1, 1])
sz = tex.size
```
