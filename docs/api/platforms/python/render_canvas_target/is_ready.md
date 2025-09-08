# is_ready() -> bool

Returns True if the underlying WindowTarget has been initialized.

This becomes True after the renderer creates and binds a platform surface to the wrapper.

## Example

```python
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
ctx = renderer.create_target(canvas)
assert isinstance(ctx.is_ready(), bool)
```
