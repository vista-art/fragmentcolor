from fragmentcolor import Renderer


renderer = Renderer()
target = renderer.create_texture_target([16, 16])
renderer.render(fragmentcolor.Shader.default(), target)

image = target.get_image()
