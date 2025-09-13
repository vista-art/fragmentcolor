
from fragmentcolor import Shader, Pass, Renderer, Frame

renderer = Renderer()
target = renderer.create_texture_target([100, 100])

pass1 = Pass("first")
pass2 = Pass("second")

frame = Frame()
frame.add_pass(pass1)
frame.add_pass(pass2)
