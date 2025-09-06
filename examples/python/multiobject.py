from fragmentcolor import Renderer, Shader, Pass
from rendercanvas.auto import RenderCanvas, loop
from pathlib import Path
import random

BASE_DIR = Path(__file__).resolve().parent
shader_file = str(BASE_DIR / "circle.wgsl")

canvas = RenderCanvas(size=(800, 600))
renderer = Renderer()
target = renderer.create_target(canvas)

renderpass = Pass("Multi Object Pass")

circles = []
for _ in range(50):
    circle = Shader(shader_file)
    circle.set("resolution", [800, 600])

    # Random position within canvas bounds
    x = random.uniform(-800, 800)
    y = random.uniform(-600, 600)
    circle.set("circle.position", [x, y])

    # Random color components
    r = random.random()
    g = random.random()
    b = random.random()
    circle.set("circle.color", [r, g, b, 1.0])

    # Random radius and border
    radius = random.uniform(50, 300)
    circle.set("circle.radius", radius)
    border = random.uniform(10, 100)
    circle.set("circle.border", border)

    renderpass.add_shader(circle)
    circles.append(circle)


@canvas.add_event_handler("resize")
def handler(event):
    if event['event_type'] == "resize":
        ratio = event["pixel_ratio"]
        w = event['width'] * ratio
        h = event['height'] * ratio
        for circle in circles:
            circle.set("resolution", [w, h])
        target.resize([w, h])


@canvas.request_draw
def animate():
    renderer.render(renderpass, target)


# Enter main rendering loop
loop.run()
