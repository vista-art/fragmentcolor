
from fragmentcolor import Renderer, Shader

renderer = Renderer()
target = renderer.create_texture_target([64, 64])

shader = Shader.default()
renderer.render(shader, target)

image = target.get_image()
