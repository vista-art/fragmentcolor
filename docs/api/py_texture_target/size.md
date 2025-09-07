# size() -> [int, int]

Returns the current size of the [PyTextureTarget](https://fragmentcolor.org/api/py_texture_target) as a `[width, height]` pair.

## Example

```python
from fragmentcolor import Renderer

renderer = Renderer()
target = renderer.create_texture_target([64, 64])
assert target.size() == [64, 64]
```
