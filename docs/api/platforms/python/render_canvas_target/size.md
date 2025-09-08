# size() -> [u32, u32]

Returns the current size in pixels of the backing WindowTarget.

## Example

```python
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
ctx = renderer.create_target(canvas)
wh = ctx.size
assert isinstance(wh, list) and len(wh) == 2
```
