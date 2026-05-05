# TextureMipChain::prepare (Python)

Build a mip chain with positional args: prepare(bytes, format, size=None).

## Example

```python
from fragmentcolor import Renderer, TextureFormat, TextureMipChain

# Raw pixel path -- positional args: prepare(bytes, format, size).
raw_rgba = [200] * (8 * 8 * 4)
chain = TextureMipChain.prepare(raw_rgba, TextureFormat.Rgba8UnormSrgb, [8, 8])

# Hand the chain to the unified create_texture entry.
renderer = Renderer()
texture = renderer.create_texture(chain)
```
