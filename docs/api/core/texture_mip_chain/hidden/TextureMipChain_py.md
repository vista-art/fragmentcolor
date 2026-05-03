# TextureMipChain (Python)

Python-specific example for creating a TextureMipChain.

## Example

```python
from fragmentcolor import Renderer, TextureFormat, TextureMipChain

renderer = Renderer()
# Minimal 1x1 RGBA raw pixel bytes.
pixels = [255, 0, 0, 255]
chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [1, 1])

# Hand the chain to the unified create_texture entry.
texture = renderer.create_texture(chain)
```
