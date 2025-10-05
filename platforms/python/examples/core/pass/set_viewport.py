
from fragmentcolor import Renderer, Pass, Shader, Region

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

shader = Shader.default()
rpass = Pass("clipped")
rpass.add_shader(shader)

rpass.set_viewport(Region((0, 0), (32, 32)))

renderer.render(rpass, target)
