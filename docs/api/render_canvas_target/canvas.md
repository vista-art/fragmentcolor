# canvas() -> RenderCanvas

Returns the underlying RenderCanvas object used to integrate with the windowing system.

## Example

```python
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
ctx = renderer.create_target(canvas)
rc = ctx.canvas
# rc is a RenderCanvas instance
```
