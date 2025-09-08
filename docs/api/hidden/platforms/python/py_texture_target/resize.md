# resize(size: [int, int])

Resizes the [PyTextureTarget](https://fragmentcolor.org/api/hidden/platforms/python/pytexturetarget) to the given `[width, height]` dimensions.

## Example

```python
from fragmentcolor import Renderer

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

target.resize([128, 128])
assert target.size() == [128, 128]
```
