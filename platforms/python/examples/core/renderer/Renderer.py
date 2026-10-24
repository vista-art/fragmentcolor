
from fragmentcolor import Shader, Renderer
from rendercanvas.auto import RenderCanvas
canvas = RenderCanvas(size=(800, 600))

renderer = Renderer()

# The canvas comes from your platform's windowing system
target = renderer.create_target(canvas)
texture_target = renderer.create_texture_target([16, 16])

# RENDERING
renderer.render(Shader.default(), texture_target)

# That's it. Welcome to FragmentColor!
