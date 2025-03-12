from fragmentcolor import FragmentColor, Shader
from rendercanvas.auto import RenderCanvas, loop

# Creates a window
canvas = RenderCanvas()
renderer, target = FragmentColor.init(canvas)

# Parses the uniforms automatically and exposes their names as keys
circle = Shader("circle.wgsl")
circle.set("circle.radius", 200.0)
circle.set("circle.color", (1.0, 0.0, 0.0, 0.8))
circle.set("circle.border", 20.0)


@canvas.resize
def resize(w, h):
    circle.set("resolution", (w, h))


@canvas.request_draw
def animate():
    circle.set("position", (0.0, 0.0))
    renderer.render(circle, target)


# Enter main rendering loop
loop.run()
