# create_target(target: Canvas | Window)

Creates a [Target](https://fragmentcolor.org/api/target) attached to a platform-specific canvas or window.

## Example

### Javascript (Web)

```js
import init, { Renderer } from "fragmentcolor";
await init();
const renderer = new Renderer();
const canvas = document.createElement("canvas");
const target = await renderer.createTarget(canvas);
```

### Python

```python
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas

renderer = Renderer()
canvas = RenderCanvas()
target = renderer.create_target(canvas)
```
