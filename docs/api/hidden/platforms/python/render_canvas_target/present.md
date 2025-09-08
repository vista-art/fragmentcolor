# present() -> dict

Presents the current frame to the screen via the underlying WindowTarget.

Returns a dict containing a presentation method (e.g., "screen") and optional error information on failure.

## Example

```python
from fragmentcolor import Renderer, Shader
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
ctx = renderer.create_target(canvas)

shader = Shader("// wgsl ...")
renderer.render(shader, ctx)

result = ctx.present()
assert isinstance(result, dict)
```
