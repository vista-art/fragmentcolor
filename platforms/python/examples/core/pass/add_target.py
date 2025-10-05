from fragmentcolor import Renderer, Pass, TextureFormat

r = Renderer()
tex_target = r.create_texture_target([512, 512])

p = Pass("shadow")
p.add_target(tex_target)
