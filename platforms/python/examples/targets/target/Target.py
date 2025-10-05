from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Renderer, Shader

renderer = Renderer()

# Use your platform's windowing system to create a window.
# We officially support Winit. Check the examples folder for details.
canvas = RenderCanvas(size=(800, 600))

target = renderer.create_target(canvas)

# To animate, render again in your event loop...
renderer.render(Shader(""), target)
renderer.render(Shader(""), target)
