from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Shader, Pass, Renderer

renderer = Renderer()
canvas = RenderCanvas(size=(100, 100))
target = renderer.create_target(canvas)
shader = Shader.default()

rpass = Pass("First Pass")
rpass.add_shader(shader)

pass2 = Pass("Second Pass")
pass2.add_shader(shader)

# standalone
renderer.render(rpass, target)

# vector of passes rendered in order (any iterable of Pass is renderable)
renderer.render([rpass, pass2], target)
