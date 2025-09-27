from fragmentcolor import Renderer, Pass

r = Renderer()
tex_target = r.create_texture_target([512, 512])

p = Pass("shadow")
p.add_target(tex_target)
