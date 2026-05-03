# Texture::set_sampler_options (Python)

Configure sampler options via a dict.

## Example

```python
from fragmentcolor import Renderer, SamplerOptions
renderer = Renderer()
# 1x1 RGBA (white) raw pixel bytes
pixels = [255, 255, 255, 255]

texture = renderer.create_texture(pixels, size=[1, 1])
opts = {"repeat_x": True, "repeat_y": True, "smooth": True, "compare": None}
texture.set_sampler_options(opts)
```
