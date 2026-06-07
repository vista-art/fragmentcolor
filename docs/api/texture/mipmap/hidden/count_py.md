# Mipmap::count (Python)

Return the number of mip levels in this chain.

## Example

```python
from fragmentcolor import TextureFormat, Mipmap

pixels = [0] * (8 * 8 * 4)
chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [8, 8])
count = chain.count()
_ = count
```
