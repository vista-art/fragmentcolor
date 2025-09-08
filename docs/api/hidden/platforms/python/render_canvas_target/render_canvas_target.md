# RenderCanvasTarget

[RenderCanvasTarget](https://fragmentcolor.org/api/hidden/platforms/python/rendercanvastarget) is a Python-specific wrapper around [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget).

It adapts [FragmentColor](https://fragmentcolor.org) to work with the RenderCanvas Python project, forwarding all rendering to an internal [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget). See [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) for the full [Target](https://fragmentcolor.org/api/core/target) behavior and semantics.

- Canonical object: [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget)
- [Target](https://fragmentcolor.org/api/core/target) trait docs: [Target](https://fragmentcolor.org/api/core/target)

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

