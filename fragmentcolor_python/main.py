from fragmentcolor import FragmentColor as fc, Shader
from rendercanvas.auto import RenderCanvas, loop

canvas = RenderCanvas(size=(800, 600))
renderer, target = fc.init(canvas)

with open("circle.wgsl") as file:
    source = file.read()

circle = Shader(source)
circle.set("resolution", [800, 600])
circle.set("circle.radius", 200.0)
circle.set("circle.color", [1.0, 0.0, 0.0, 0.8])
circle.set("circle.border", 20.0)

renderer.render(circle, target)


@canvas.resize
def resize(w, h):
    circle.set("resolution", (w, h))


@canvas.request_draw
def animate():
    circle.set("position", [0.0, 0.0])
    renderer.render(circle, target)


# Enter main rendering loop
loop.run()
