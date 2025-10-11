from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Renderer

renderer = Renderer()

# Use your platform's windowing system to create a window.
canvas = RenderCanvas(size=(800, 600))

target = renderer.create_target(canvas)
