
from fragmentcolor import Renderer, Shader
import os, sys
# Skip this example in headless/CI environments that have no display surface.
if os.environ.get('DISPLAY') is None and sys.platform != 'win32' and os.environ.get('FC_ALLOW_WINDOW') != '1':
    raise SystemExit(0)
from rendercanvas.auto import RenderCanvas
canvas = RenderCanvas(size=(800, 600))

renderer = Renderer()

# The canvas comes from your platform's windowing system.
target = renderer.create_target(canvas)

# To animate, render again in your event loop...
renderer.render(Shader.default(), target)
renderer.render(Shader.default(), target)
