# WindowTarget.size (Python)

Python override for `WindowTarget::size`. The Python window target reads its
size with a `size()` call, while `Texture` and `TextureTarget` expose `size` as
a property, so this page keeps the call that the generic Python conversion
would turn into a property read.

## Example

```python
import os, sys
# Skip this example in headless/CI environments that have no display surface.
if os.environ.get('DISPLAY') is None and sys.platform != 'win32' and os.environ.get('FC_ALLOW_WINDOW') != '1':
    raise SystemExit(0)
from rendercanvas.auto import RenderCanvas
from fragmentcolor import Renderer

canvas = RenderCanvas(size=(800, 600))
renderer = Renderer()
target = renderer.create_target(canvas)
size = target.size()
```
