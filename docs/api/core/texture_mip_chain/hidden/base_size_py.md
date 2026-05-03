# TextureMipChain::base_size (Python)

Return the base level dimensions as a (width, height) tuple.

## Example

```python
from fragmentcolor import TextureFormat, TextureMipChain

pixels = [0] * (16 * 16 * 4)
chain = TextureMipChain.prepare(pixels, TextureFormat.Rgba8UnormSrgb, [16, 16])
(width, height) = chain.base_size()
_ = (width, height)
```
