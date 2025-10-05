
from fragmentcolor import Renderer, Pass, Shader

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

shader = Shader.default()
rpass = Pass("blend with previous")
rpass.add_shader(shader)
rpass.load_previous()

renderer.render(rpass, target)
