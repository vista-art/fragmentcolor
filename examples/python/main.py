from fragmentcolor import Renderer, Shader
from rendercanvas.auto import RenderCanvas, loop
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent
shader_file = str(BASE_DIR / "circle.wgsl")

canvas = RenderCanvas(size=(800, 600))
renderer = Renderer()
target = renderer.create_target(canvas)

circle = Shader(shader_file)
circle.set("resolution", [800, 600])
circle.set("circle.radius", 200.0)
circle.set("circle.color", [1.0, 0.0, 0.0, 0.8])
circle.set("circle.border", 20.0)

print(circle.get("circle.radius"))
print(circle.get("circle.color"))
print(circle.get("circle.border"))
print(circle.get("resolution"))


@canvas.add_event_handler("resize")
def handler(event):
    if event['event_type'] == "resize":
        ratio = event["pixel_ratio"]
        w = event['width'] * ratio
        h = event['height'] * ratio
        circle.set("resolution", [w, h])
        target.resize([w, h])


@canvas.request_draw
def animate():
    circle.set("circle.position", [0.0, 0.0])
    renderer.render(circle, target)


# Enter main rendering loop
loop.run()
