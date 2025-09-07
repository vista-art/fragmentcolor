# resize(size: [u32, u32] | (u32, u32) | { width, height })

Resizes the underlying WindowTarget to the given dimensions.

## Example

```python
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
ctx = renderer.create_target(canvas)
ctx.resize([640, 480])
```
