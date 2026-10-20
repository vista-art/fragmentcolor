
from fragmentcolor import Shader, Pass, Renderer
import os, sys
# Skip this example in headless/CI environments that have no display surface.
if os.environ.get('DISPLAY') is None and sys.platform != 'win32' and os.environ.get('FC_ALLOW_WINDOW') != '1':
    raise SystemExit(0)
from rendercanvas.auto import RenderCanvas
canvas = RenderCanvas(size=(100, 100))

renderer = Renderer()
target = renderer.create_target(canvas)
shader = Shader.default()

rpass = Pass("First Pass")
rpass.add_shader(shader)

pass2 = Pass("Second Pass")
pass2.add_shader(shader)

# standalone
renderer.render(rpass, target)

# vector of passes rendered in order (any iterable of Pass is renderable)
renderer.render([rpass, pass2], target)
