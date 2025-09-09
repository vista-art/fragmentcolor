
from fragmentcolor import Renderer, Pass, Shader

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

shader = Shader.default()
rpass = Pass("blend with previous")
pass.add_shader(shader)
pass.load_previous()

renderer.render(pass, target)
