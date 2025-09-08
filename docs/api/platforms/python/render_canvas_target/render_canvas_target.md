# RenderCanvasTarget

[RenderCanvasTarget](https://fragmentcolor.org/api/render_canvas_target) is a Python-specific wrapper around [WindowTarget](https://fragmentcolor.org/api/window_target).

It adapts [FragmentColor](https://fragmentcolor.org) to work with the RenderCanvas Python project, forwarding all rendering to an internal [WindowTarget](https://fragmentcolor.org/api/window_target). See [WindowTarget](https://fragmentcolor.org/api/window_target) for the full [Target](https://fragmentcolor.org/api/target) behavior and semantics.

- Canonical object: [WindowTarget](https://fragmentcolor.org/api/window_target)
- Target trait docs: [Target](https://fragmentcolor.org/api/target)

## Example

```python
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
# RenderCanvas calls the fragmentcolor hook to create a RenderCanvasTarget under the hood
# via renderer.create_target(canvas)

target = renderer.create_target(canvas)  # returns RenderCanvasTarget
# You can now render Frames, Passes or Shaders to this target
```

