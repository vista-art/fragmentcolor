# Texture::set_sampler_options (Python)

Takes a `SamplerOptions`, or a dict with the same keys.

## Example

```python
from fragmentcolor import Renderer, SamplerOptions
renderer = Renderer()
# 1x1 RGBA (white) raw pixel bytes
pixels = [255, 255, 255, 255]

texture = renderer.create_texture(pixels, size=[1, 1])
opts = SamplerOptions(repeat_x=True, repeat_y=True, smooth=True, compare=None)
texture.set_sampler_options(opts)
```
