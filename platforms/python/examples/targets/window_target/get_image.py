from fragmentcolor import Renderer, Shader
import os, sys
# Skip this example in headless/CI environments that have no display surface.
if os.environ.get('DISPLAY') is None and sys.platform != 'win32' and os.environ.get('FC_ALLOW_WINDOW') != '1':
    raise SystemExit(0)
from rendercanvas.auto import RenderCanvas
canvas = RenderCanvas(size=(800, 600))

renderer = Renderer()
target = renderer.create_target(canvas)
shader = Shader.default()
renderer.render(shader, target)

# The same frame, drawn where it can be read back.
offscreen = renderer.create_texture_target([800, 600])
renderer.render(shader, offscreen)
image = offscreen.get_image()