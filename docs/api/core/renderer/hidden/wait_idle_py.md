# Renderer::wait_idle()

Python wrapper for `Renderer::wait_idle`. Blocks until all GPU submissions have finished.

## Example

```python
from fragmentcolor import Renderer, Shader

renderer = Renderer()
target = renderer.create_texture_target((8, 8))
shader = Shader("void main() { fragColor = vec4(1.0); }")
renderer.render(shader, target)
renderer.wait_idle()
bytes = target.get_image()
```
