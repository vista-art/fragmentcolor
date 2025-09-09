
from fragmentcolor import Renderer, Pass, Shader, Region

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

shader = Shader.default()
rpass = Pass("clipped")
pass.add_shader(shader)

pass.set_viewport(Region.from_region(0, 0, 32, 32))

renderer.render(pass, target)
