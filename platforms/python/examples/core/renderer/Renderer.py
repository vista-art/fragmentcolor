from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Shader, Renderer

renderer = Renderer()

# Use your platform's windowing system to create a window
canvas = RenderCanvas(size=(800, 600))

# Create a Target from it
target = renderer.create_target(canvas)
texture_target = renderer.create_texture_target([16, 16])

# RENDERING
renderer.render(Shader(""), texture_target)

# That's it. Welcome to FragmentColor!
