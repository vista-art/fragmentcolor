
from fragmentcolor import Renderer, Pass, Shader

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

shader = Shader.default()
rpass = Pass("solid background")
pass.add_shader(shader)

pass.set_clear_color([0.1, 0.2, 0.3, 1.0])

renderer.render(pass, target)
