import os, sys
# Skip this example in headless/CI environments that have no display surface.
if os.environ.get('DISPLAY') is None and sys.platform != 'win32' and os.environ.get('FC_ALLOW_WINDOW') != '1':
    raise SystemExit(0)

from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Renderer, Shader

# Use your platform's windowing system to create a window.
canvas = RenderCanvas(size=(800, 600))

renderer = Renderer()
target = renderer.create_target(canvas)

renderer.render(Shader(""), target)