# Mipmap::level (Python)

Return the bytes for a single mip level by index.

## Example

```python
from fragmentcolor import TextureFormat, Mipmap

pixels = [0] * (8 * 8 * 4)
chain = Mipmap.build(pixels, TextureFormat.Rgba8UnormSrgb, [8, 8])
level_zero_bytes = chain.level(0)
_ = level_zero_bytes
```
