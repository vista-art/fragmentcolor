# RenderCanvasTarget.new()

Creates a new [RenderCanvasTarget](https://fragmentcolor.org/api/hidden/platforms/python/rendercanvastarget) wrapper, typically invoked by RenderCanvas when requesting a "fragmentcolor" context.

The object wraps a platform [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) internally and exposes the [Target](https://fragmentcolor.org/api/core/target) interface in Python.

## Example

```python
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
# Under the hood this calls the hook and constructs RenderCanvasTarget
target = renderer.create_target(canvas)
```
