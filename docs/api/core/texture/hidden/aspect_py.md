# Texture::aspect (Python)

Return the width-to-height ratio of this texture.

## Example

```python
from fragmentcolor import Renderer

renderer = Renderer()
# 1x1 RGBA (white) raw pixel bytes
pixels = [255, 255, 255, 255]
tex = renderer.create_texture(pixels, size=[1, 1])
a = tex.aspect()
```
