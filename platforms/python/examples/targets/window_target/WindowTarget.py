from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Renderer, Shader

# Use your platform's windowing system to create a window.
canvas = RenderCanvas(size=(800, 600))

renderer = Renderer()
target = renderer.create_target(canvas)

renderer.render(Shader(""), target)
