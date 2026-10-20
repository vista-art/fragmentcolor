
from fragmentcolor import Renderer
from rendercanvas.auto import RenderCanvas
canvas = RenderCanvas(size=(800, 600))

renderer = Renderer()

# The canvas comes from your platform's windowing system.
target = renderer.create_target(canvas)
