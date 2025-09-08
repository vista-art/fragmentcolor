# RenderCanvasTarget.new()

Creates a new [RenderCanvasTarget](https://fragmentcolor.org/api/render_canvas_target) wrapper, typically invoked by RenderCanvas when requesting a "fragmentcolor" context.

The object wraps a platform [WindowTarget](https://fragmentcolor.org/api/window_target) internally and exposes the [Target](https://fragmentcolor.org/api/target) interface in Python.

## Example

```python
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
# Under the hood this calls the hook and constructs RenderCanvasTarget
target = renderer.create_target(canvas)
```
