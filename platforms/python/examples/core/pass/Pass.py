from rendercanvas.auto import RenderCanvas, loop

from fragmentcolor import Shader, Pass, Renderer, Frame

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

# using a Frame
frame = Frame()
frame.add_pass(rpass)
frame.add_pass(pass2)
renderer.render(frame, target)

# vector of passes (consume them)
renderer.render([rpass, pass2], target)
