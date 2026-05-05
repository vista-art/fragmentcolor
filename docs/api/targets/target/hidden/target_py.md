# Target (Python)

Python-specific example for creating a Target from a RenderCanvas. Requires a windowing system.

## Example

```python
import os, sys
# Skip this example in headless/CI environments that have no display surface.
if os.environ.get('DISPLAY') is None and sys.platform != 'win32' and os.environ.get('FC_ALLOW_WINDOW') != '1':
    raise SystemExit(0)

from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Renderer, Shader

renderer = Renderer()

# Use your platform's windowing system to create a window.
canvas = RenderCanvas(size=(800, 600))

target = renderer.create_target(canvas)

# To animate, render again in your event loop...
renderer.render(Shader(""), target)
renderer.render(Shader(""), target)
```
