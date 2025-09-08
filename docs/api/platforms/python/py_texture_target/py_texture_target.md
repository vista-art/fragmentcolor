# PyTextureTarget

Python wrapper around the headless [TextureTarget](https://fragmentcolor.org/api/texture_target). It implements the [Target](https://fragmentcolor.org/api/target) interface and is returned by `Renderer.create_texture_target(...)` in Python.

Use this when you need an offscreen render target to read back pixels or render without a window.

## Example

```python
from fragmentcolor import Renderer

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

w, h = target.size()
assert (w, h) == (64, 64)

# Resize
target.resize([128, 128])
assert target.size() == [128, 128]
```
