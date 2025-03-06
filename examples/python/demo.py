import fragmentcolor as fc
from rendercanvas.auto import RenderCanvas, loop

# Creates a window
canvas = RenderCanvas()
res = canvas.get_physical_size()

# Creates the shader with default values
shader = fc.Shader("gaze.json")

# It parses and binds automatically
shader.set("resolution", res)

# Create renderer
renderer = fc.Renderer()


@canvas.resize
def resize(w, h):
    shader.set("resolution", (w, h))


@canvas.request_draw
def animate():
    shader.set("circle.position", (0.0, 0.0))
    renderer.render(shader, canvas)


# Enter main rendering loop
loop.run()
